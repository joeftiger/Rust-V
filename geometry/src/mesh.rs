use crate::bvh::Tree;
use crate::debug_util::is_finite;
use crate::obj_file::ObjFile;
use crate::{Aabb, Boundable, Geometry, Intersectable, Intersection, Ray};
use itertools::{Itertools, MinMaxResult};
use serde::de::{Error, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Debug;
use std::mem::swap;
use std::path::Path;
use ultraviolet::{Mat3, Rotor3, Vec3};
#[cfg(feature = "watertight-mesh")]
use utility::floats::fast_clamp;
#[cfg(not(feature = "watertight-mesh"))]
use utility::floats::{fast_clamp, in_range, EPSILON};

/// The shading mode defines the shading of normals. In `Flat` mode, the surface of triangles will
/// appear flat. In `Phong` however, they will be interpolated to create a smooth looking surface.
#[derive(Debug, Serialize, Deserialize)]
pub enum ShadingMode {
    Flat,
    Phong,
}

/// Returns the index of the maximum component of a vector.
#[inline(always)]
fn max_index(v: &Vec3) -> usize {
    if v.x > v.y {
        if v.x > v.z {
            return 0;
        }
    } else if v.y > v.z {
        return 1;
    }

    2
}

/// A triangle consists of vertex indices `(v0, v1, v2)` and their optional normal indices `n0, n1, n2)`.
///
/// In order to query a triangle for an intersection, it is therefore needed to pass it the proper mesh.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Face {
    pub v: (u32, u32, u32),
    pub vn: Option<(u32, u32, u32)>,
}

impl Face {
    pub fn new(v: (u32, u32, u32), vn: Option<(u32, u32, u32)>) -> Self {
        Self { v, vn }
    }

    pub fn get_vertices(&self, vertices: &[Vec3]) -> (Vec3, Vec3, Vec3) {
        (
            vertices[self.v.0 as usize],
            vertices[self.v.1 as usize],
            vertices[self.v.2 as usize],
        )
    }

    pub fn get_normals(&self, normals: &[Vec3]) -> Option<(Vec3, Vec3, Vec3)> {
        if let Some((n0, n1, n2)) = self.vn {
            Some((
                normals[n0 as usize],
                normals[n1 as usize],
                normals[n2 as usize],
            ))
        } else {
            None
        }
    }

    pub fn face_normal(&self, vertices: &[Vec3]) -> Vec3 {
        let (v0, v1, v2) = self.get_vertices(vertices);

        (v1 - v0).cross(v2 - v0).normalized()
    }

    pub fn has_normals(&self) -> bool {
        self.vn.is_some()
    }

    pub fn bounds(&self, vertices: &[Vec3]) -> Aabb {
        let (v0, v1, v2) = self.get_vertices(vertices);

        let min = v0.min_by_component(v1.min_by_component(v2));
        let max = v0.max_by_component(v1.max_by_component(v2));

        Aabb::new(min, max)
    }

    #[cfg(feature = "watertight-mesh")]
    #[allow(clippy::many_single_char_names)]
    fn intersect(&self, mesh: &Mesh, ray: &Ray) -> Option<Intersection> {
        let (v0, v1, v2) = self.get_vertices(&mesh.vertices);

        let dir = &ray.direction;
        // calculate dimension where the ray direction is maximal
        let kz = max_index(&dir.abs());
        let mut kx = kz + 1;
        if kx == 3 {
            kx = 0
        }
        let mut ky = kx + 1;
        if ky == 3 {
            ky = 0
        }

        // swap dimension to preserve winding direction of triangles
        if dir[kz] < 0.0 {
            swap(&mut kx, &mut ky);
        }

        // calculate shear constants
        let sx = dir[kx] / dir[kz];
        let sy = dir[ky] / dir[kz];
        let sz = 1.0 / dir[kz];

        // calculate vertices relative to ray origin
        let a = v0 - ray.origin;
        let b = v1 - ray.origin;
        let c = v2 - ray.origin;

        // perform shear and scale of vertices
        let ax = a[kx] - sx * a[kz];
        let ay = a[ky] - sy * a[kz];
        let bx = b[kx] - sx * b[kz];
        let by = b[ky] - sy * b[kz];
        let cx = c[kx] - sx * c[kz];
        let cy = c[ky] - sy * c[kz];

        // calculate scaled barycentric coordinates
        let mut u = cx * by - cy * bx;
        let mut v = ax * cy - ay * cx;
        let mut w = bx * ay - by * ax;

        // perform edge tests
        if u < 0.0 || v < 0.0 || w < 0.0 {
            return None;
        }

        // fallback to test against edges using double precision
        if u == 0.0 {
            u = (cx as f64 * by as f64 - cy as f64 * bx as f64) as f32;
        }
        if v == 0.0 {
            v = (ax as f64 * cy as f64 - ay as f64 * cx as f64) as f32;
        }
        if w == 0.0 {
            w = (bx as f64 * ay as f64 - by as f64 * ax as f64) as f32;
        }

        // perform edge tests
        if u < 0.0 || v < 0.0 || w < 0.0 {
            return None;
        }

        // calculate determinant
        let det = u + v + w;
        if det == 0.0 {
            return None;
        }

        // for normalization
        let inv_det = 1.0 / det;

        // calculate scaled z-coordinates of vertices and use them to calculate the hit distance
        let az = sz * a[kz];
        let bz = sz * b[kz];
        let cz = sz * c[kz];
        let t = (u * az + v * bz + w * cz) * inv_det;

        if !ray.contains(t) {
            return None;
        }

        let point = ray.at(t);

        let normal = match mesh.shading_mode {
            ShadingMode::Flat => (v1 - v0).cross(v2 - v0),
            ShadingMode::Phong => {
                if let Some((n0, n1, n2)) = self.get_normals(&mesh.vertex_normals) {
                    let beta = u * inv_det;
                    let gamma = v * inv_det;
                    let alpha = 1.0 - beta - gamma;

                    alpha * n0 + beta * n1 + gamma * n2
                } else {
                    (v1 - v0).cross(v2 - v0)
                }
            }
        }
        .normalized();

        Some(Intersection::new(point, normal, t, *ray))
    }

    #[cfg(not(feature = "watertight-mesh"))]
    fn intersect(&self, mesh: &Mesh, ray: &Ray) -> Option<Intersection> {
        let (v0, v1, v2) = self.get_vertices(&mesh.vertices);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        // ray is parallel to triangle
        if in_range(a, -EPSILON, EPSILON) {
            return None;
        }

        let f = 1.0 / a;
        let s = ray.origin - v0;
        let beta = f * s.dot(h);
        if beta < 0.0 || beta > 1.0 {
            return None;
        }

        let q = s.cross(edge1);
        let gamma = f * ray.direction.dot(q);
        if gamma < 0.0 || beta + gamma > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);
        if !ray.contains(t) {
            return None;
        }

        let point = ray.at(t);

        let normal = match mesh.shading_mode {
            ShadingMode::Flat => edge1.cross(edge2),
            ShadingMode::Phong => {
                if let Some((n0, n1, n2)) = self.get_normals(&mesh.vertex_normals) {
                    let beta = u * inv_det;
                    let gamma = v * inv_det;
                    let alpha = 1.0 - beta - gamma;

                    alpha * n0 + beta * n1 + gamma * n2
                } else {
                    edge1.cross(edge2)
                }
            }
        }
        .normalized();

        Some(Intersection::new(point, normal, t, *ray))
    }

    #[cfg(feature = "watertight-mesh")]
    #[allow(clippy::many_single_char_names)]
    fn intersects(&self, vertices: &[Vec3], ray: &Ray) -> bool {
        let (v0, v1, v2) = self.get_vertices(vertices);

        let dir = &ray.direction;
        // calculate dimension where the ray direction is maximal
        let kz = max_index(&dir.abs());
        let mut kx = kz + 1;
        if kx == 3 {
            kx = 0
        }
        let mut ky = kx + 1;
        if ky == 3 {
            ky = 0
        }

        // swap dimension to preserve winding direction of triangles
        if dir[kz] < 0.0 {
            swap(&mut kx, &mut ky);
        }

        // calculate shear constants
        let sx = dir[kx] / dir[kz];
        let sy = dir[ky] / dir[kz];
        let sz = 1.0 / dir[kz];

        // calculate vertices relative to ray origin
        let a = v0 - ray.origin;
        let b = v1 - ray.origin;
        let c = v2 - ray.origin;

        // perform shear and scale of vertices
        let ax = a[kx] - sx * a[kz];
        let ay = a[ky] - sy * a[kz];
        let bx = b[kx] - sx * b[kz];
        let by = b[ky] - sy * b[kz];
        let cx = c[kx] - sx * c[kz];
        let cy = c[ky] - sy * c[kz];

        // calculate scaled barycentric coordinates
        let mut u = cx * by - cy * bx;
        let mut v = ax * cy - ay * cx;
        let mut w = bx * ay - by * ax;

        // perform edge tests
        if u < 0.0 || v < 0.0 || w < 0.0 {
            return false;
        }

        // fallback to test against edges using double precision
        if u == 0.0 {
            u = (cx as f64 * by as f64 - cy as f64 * bx as f64) as f32;
        }
        if v == 0.0 {
            v = (ax as f64 * cy as f64 - ay as f64 * cx as f64) as f32;
        }
        if w == 0.0 {
            w = (bx as f64 * ay as f64 - by as f64 * ax as f64) as f32;
        }

        // perform edge tests
        if u < 0.0 || v < 0.0 || w < 0.0 {
            return false;
        }

        // calculate determinant
        let det = u + v + w;
        if det == 0.0 {
            return false;
        }

        // for normalization
        let inv_det = 1.0 / det;

        // calculate scaled z-coordinates of vertices and use them to calculate the hit distance
        let az = sz * a[kz];
        let bz = sz * b[kz];
        let cz = sz * c[kz];
        let t = (u * az + v * bz + w * cz) * inv_det;

        ray.contains(t)
    }

    #[cfg(not(feature = "watertight-mesh"))]
    fn intersects(&self, vertices: &[Vec3], ray: &Ray) -> bool {
        let (v0, v1, v2) = self.get_vertices(vertices);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        // ray is parallel to triangle
        if in_range(a, -EPSILON, EPSILON) {
            return false;
        }

        let f = 1.0 / a;
        let s = ray.origin - v0;
        let beta = f * s.dot(h);
        if beta < 0.0 || beta > 1.0 {
            return false;
        }

        let q = s.cross(edge1);
        let gamma = f * ray.direction.dot(q);
        if gamma < 0.0 || beta + gamma > 1.0 {
            return false;
        }

        let t = f * edge2.dot(q);

        ray.contains(t)
    }
}

/// A mesh consists of vertices and triangles, allowing queries for intersections.
/// Depending on the [`MeshMode`](MeshMode), the intersection normals will be interpolated.
pub struct Mesh {
    vertices: Vec<Vec3>,
    vertex_normals: Vec<Vec3>,
    faces: Vec<Face>,
    bounds: Aabb,
    shading_mode: ShadingMode,
    bvh: Tree<Face>,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vec3>,
        vertex_normals: Vec<Vec3>,
        faces: Vec<Face>,
        bounds: Aabb,
        shading_mode: ShadingMode,
    ) -> Self {
        Self {
            vertices,
            vertex_normals,
            faces,
            bounds,
            shading_mode,
            bvh: Default::default(),
        }
    }

    /// Loads the given tobj mesh. If the tobj mesh contains vertex normals, they will be used.
    /// Otherwise they will be computed.
    ///
    /// # Arguments
    /// * `tobj_mesh` - The tobj mesh to load
    /// * `shading_mode` - The shading mode
    ///
    /// # Returns
    /// * Self
    pub fn load<P>(path: P, shading_mode: ShadingMode) -> Mesh
    where
        P: AsRef<Path> + Debug,
    {
        let obj_file = ObjFile::from(path);

        let bounds = match obj_file.vertices.iter().minmax_by(|a, b| {
            if a.min_by_component(**b) == **a {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }) {
            MinMaxResult::NoElements => Aabb::empty(),
            MinMaxResult::OneElement(v) => Aabb::new(*v, *v),
            MinMaxResult::MinMax(min, max) => Aabb::new(*min, *max),
        };

        Mesh::new(
            obj_file.vertices,
            obj_file.vertex_normals,
            obj_file.faces,
            bounds,
            shading_mode,
        )
    }

    /// Determines the weights by which to scale triangle (p0, p1, p2)'s normal when
    /// accumulating the vertex normal for vertices 0, 1, 2.
    ///
    /// # Constraints
    /// * `p0` - All values should be finite (neither infinite nor `NaN`).
    /// * `p1` - All values should be finite.
    /// * `p2` - All values should be finite.
    ///
    /// # Arguments
    /// * `p0` - The position 0 of a triangle
    /// * `p1` - The position 1 of a triangle
    /// * `p2` - The position 2 of a triangle
    ///
    /// # Returns
    /// * `w0` - The weight for vertex 0
    /// * `w1` - The weight for vertex 1
    /// * `w2` - The weight for vertex 2
    fn angle_weights(p0: Vec3, p1: Vec3, p2: Vec3) -> (f32, f32, f32) {
        debug_assert!(is_finite(&p0));
        debug_assert!(is_finite(&p1));
        debug_assert!(is_finite(&p2));

        let e01 = (p1 - p0).normalized();
        let e12 = (p2 - p1).normalized();
        let e20 = (p0 - p2).normalized();

        let w0 = fast_clamp(e01.dot(-e20), -1.0, 1.0);
        let w1 = fast_clamp(e12.dot(-e01), -1.0, 1.0);
        let w2 = fast_clamp(e20.dot(-e12), -1.0, 1.0);

        (w0, w1, w2)
    }

    /// Translates the vertices of this mesh, updating the `bounds`.
    ///
    /// For more complex operations, look at [`transform`](Mesh::transform).
    ///
    /// # Constraints
    /// * `translation` - All values should be finite (neither infinite nor `NaN`).
    ///
    /// # Arguments
    /// * `translation` - The translation for each vertex
    ///
    /// # Returns
    /// * Self for chained transformations.
    pub fn translate(&mut self, translation: Vec3) -> &mut Self {
        debug_assert!(is_finite(&translation));

        self.vertices.iter_mut().for_each(|v| *v += translation);
        self.bounds.min += translation;
        self.bounds.max += translation;

        self
    }

    /// Scales the vertices of this mesh, updating the `bounds`.
    ///
    /// For more complex operations, look at [`transform`](Mesh::transform).
    ///
    /// # Constraints
    /// * `scale` - All values should be finite (neither infinite nor `NaN`).
    ///
    /// # Arguments
    /// * `scale` - The scale for each vertex
    ///
    /// # Returns
    /// * Self for chained transformations.
    pub fn scale(&mut self, scale: Vec3) -> &mut Self {
        debug_assert!(is_finite(&scale));

        for v in &mut self.vertices {
            *v *= scale;
        }

        for n in &mut self.vertex_normals {
            *n *= scale;
            n.normalize();
        }

        self.bounds.min *= scale;
        self.bounds.max *= scale;

        self
    }

    /// Rotates the vertices of this mesh, updating the `bounds`.
    ///
    /// For more complex operations, look at [`transform`](Mesh::transform).
    ///
    /// # Constraints
    /// * `rotation` - Should be valid (TODO: clarify).
    ///
    /// # Arguments
    /// * `rotation` - The rotation for each vertex
    ///
    /// # Returns
    /// * Self for chained transformations.
    pub fn rotate(&mut self, rotation: Rotor3) -> &mut Self {
        self.transform(rotation.into_matrix())
    }

    /// Applies a transformation for the vertices of this mesh, updating the `bounds`.
    ///
    /// # Constraints
    /// * `transformation` - Should be valid (TODO: clarify).
    ///
    /// # Arguments
    /// * `transformation` - The transformation for each vertex
    ///
    /// # Returns
    /// * Self for chained transformations.
    pub fn transform(&mut self, transformation: Mat3) -> &mut Self {
        let mut new_bounds = Aabb::empty();
        self.vertices.iter_mut().for_each(|v| {
            *v = transformation * *v;

            new_bounds.min = new_bounds.min.min_by_component(*v);
            new_bounds.max = new_bounds.min.max_by_component(*v);
        });

        self.bounds = new_bounds;

        self
    }

    pub fn update_bounds(&mut self) {
        let mut new_bounds = Aabb::empty();
        self.vertices.iter_mut().for_each(|v| {
            new_bounds.min = new_bounds.min.min_by_component(*v);
            new_bounds.max = new_bounds.min.max_by_component(*v);
        });

        self.bounds = new_bounds;
    }

    pub fn build_bvh(&mut self) {
        self.bvh = Tree::new(self.faces.clone(), |f| f.bounds(&self.vertices));
    }
}

impl Boundable for Mesh {
    fn bounds(&self) -> Aabb {
        self.bounds
    }
}

impl Intersectable for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let mut new_ray = *ray;
        let mut intersection = None;

        let hits = self.bvh.intersect(ray);
        for hit in &hits {
            if let Some(i) = hit.intersect(self, &new_ray) {
                new_ray.t_end = i.t;
                intersection = Some(i);
            }
        }

        if let Some(mut i) = intersection {
            i.ray = *ray;
            Some(i)
        } else {
            None
        }
    }

    fn intersects(&self, ray: &Ray) -> bool {
        self.bvh
            .intersect(ray)
            .iter()
            .any(|t| t.intersects(&self.vertices, ray))
    }
}

#[typetag::serde]
impl Geometry for Mesh {}

impl Serialize for Mesh {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Mesh", 5)?;
        state.serialize_field("Vertices", &self.vertices)?;
        state.serialize_field("VertexNormals", &self.vertex_normals)?;
        state.serialize_field("Faces", &self.faces)?;
        state.serialize_field("Bounds", &self.bounds)?;
        state.serialize_field("ShadingMode", &self.shading_mode)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Vertices,
            VertexNormals,
            Faces,
            Bounds,
            ShadingMode,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(
                            "`Vertices`, `VertexNormals`, `Faces`, `Bounds` or `ShadingMode`",
                        )
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        match v {
                            "Vertices" => Ok(Field::Vertices),
                            "VertexNormals" => Ok(Field::VertexNormals),
                            "Faces" => Ok(Field::Faces),
                            "Bounds" => Ok(Field::Bounds),
                            "ShadingMode" => Ok(Field::ShadingMode),
                            _ => Err(de::Error::unknown_field(v, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct MeshVisitor;

        impl<'de> Visitor<'de> for MeshVisitor {
            type Value = Mesh;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Mesh")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut vertices = None;
                let mut vertex_normals = None;
                let mut triangles = None;
                let mut bounds = None;
                let mut shading_mode = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Vertices => {
                            if vertices.is_some() {
                                return Err(de::Error::duplicate_field("Vertices"));
                            } else {
                                vertices = Some(map.next_value()?);
                            }
                        }
                        Field::VertexNormals => {
                            if vertex_normals.is_some() {
                                return Err(de::Error::duplicate_field("VertexNormals"));
                            } else {
                                vertex_normals = Some(map.next_value()?);
                            }
                        }
                        Field::Faces => {
                            if triangles.is_some() {
                                return Err(de::Error::duplicate_field("Faces"));
                            } else {
                                triangles = Some(map.next_value()?);
                            }
                        }
                        Field::Bounds => {
                            if bounds.is_some() {
                                return Err(de::Error::duplicate_field("Bounds"));
                            } else {
                                bounds = Some(map.next_value()?);
                            }
                        }
                        Field::ShadingMode => {
                            if shading_mode.is_some() {
                                return Err(de::Error::duplicate_field("ShadingMode"));
                            } else {
                                shading_mode = Some(map.next_value()?);
                            }
                        }
                    }
                }
                let vertices = vertices.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let vertex_normals = vertex_normals.unwrap_or_default();
                let faces = triangles.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let bounds = bounds.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let shading_mode =
                    shading_mode.ok_or_else(|| de::Error::invalid_length(0, &self))?;

                Ok(Mesh::new(
                    vertices,
                    vertex_normals,
                    faces,
                    bounds,
                    shading_mode,
                ))
            }
        }

        const FIELDS: &[&str] = &[
            "Vertices",
            "VertexNormals",
            "Faces",
            "Bounds",
            "ShadingMode",
        ];
        deserializer
            .deserialize_struct("Mesh", FIELDS, MeshVisitor)
            .map(|mut m| {
                m.build_bvh();
                m
            })
    }
}
