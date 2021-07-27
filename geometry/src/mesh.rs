use crate::bvh::Tree;
use crate::debug_util::is_finite;
use crate::obj_file::ObjFile;
#[allow(unused_imports)]
use crate::*;
use serde::de::{Error, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Debug;
#[cfg(feature = "watertight-mesh")]
use std::mem::swap;
use std::path::Path;
#[cfg(not(feature = "watertight-mesh"))]
use utility::floats::FloatExt;

/// The shading mode defines the shading of normals. In `Flat` mode, the surface of triangles will
/// appear flat. In `Phong` however, they will be interpolated to create a smooth looking surface.
#[derive(Debug, Serialize, Deserialize)]
pub enum ShadingMode {
    Flat,
    Phong,
}

/// Returns the index of the maximum component of a vector.
#[inline]
#[cfg(feature = "watertight-mesh")]
fn max_index(v: Vector3) -> usize {
    if v.x > v.y {
        if v.x > v.z {
            return 0;
        }
    } else if v.y > v.z {
        return 1;
    }

    2
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Vertex {
    pub position: Vector3,
    #[serde(default)]
    pub normal: Vector3,
}

/// A triangle consists of vertex indices `(v0, v1, v2)`.
///
/// In order to query a triangle for an intersection, it is therefore needed to pass it the proper mesh.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Face {
    pub v: (u32, u32, u32),
    pub normal: Vector3,
}

impl Face {
    pub fn new(v: (u32, u32, u32), normal: Vector3) -> Self {
        Self { v, normal }
    }

    #[inline]
    pub fn get_vertices<'a>(&self, vertices: &'a [Vertex]) -> (&'a Vertex, &'a Vertex, &'a Vertex) {
        (
            &vertices[self.v.0 as usize],
            &vertices[self.v.1 as usize],
            &vertices[self.v.2 as usize],
        )
    }

    pub fn bounds(&self, vertices: &[Vertex]) -> Aabb {
        let (v0, v1, v2) = self.get_vertices(vertices);
        let (v0, v1, v2) = (v0.position, v1.position, v2.position);

        let min = v0.min_by_component(v1.min_by_component(v2));
        let max = v0.max_by_component(v1.max_by_component(v2));

        Aabb::new(min, max)
    }

    #[cfg(feature = "watertight-mesh")]
    #[allow(clippy::many_single_char_names)]
    fn intersect(&self, mesh: &Mesh, ray: &Ray) -> Option<Intersection> {
        let (v0, v1, v2) = self.get_vertices(&mesh.vertices);
        let (p0, p1, p2) = (v0.position, v1.position, v2.position);

        let dir = &ray.direction;
        // calculate dimension where the ray direction is maximal
        let kz = max_index(dir.abs());
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
        let a = p0 - ray.origin;
        let b = p1 - ray.origin;
        let c = p2 - ray.origin;

        // perform shear and scale of vertices
        let ax = a[kx] - sx * a[kz];
        let ay = a[ky] - sy * a[kz];
        let bx = b[kx] - sx * b[kz];
        let by = b[ky] - sy * b[kz];
        let cx = c[kx] - sx * c[kz];
        let cy = c[ky] - sy * c[kz];

        // calculate scaled barycentric coordinates
        #[allow(unused_mut)]
        let mut u = cx * by - cy * bx;
        #[allow(unused_mut)]
        let mut v = ax * cy - ay * cx;
        #[allow(unused_mut)]
        let mut w = bx * ay - by * ax;

        // perform edge tests
        if u < 0.0 || v < 0.0 || w < 0.0 {
            return None;
        }

        // fallback to test against edges using double precision
        #[cfg(not(feature = "f64"))]
        {
            if u == 0.0 {
                u = (cx as f64 * by as f64 - cy as f64 * bx as f64) as Float;
            }
            if v == 0.0 {
                v = (ax as f64 * cy as f64 - ay as f64 * cx as f64) as Float;
            }
            if w == 0.0 {
                w = (bx as f64 * ay as f64 - by as f64 * ax as f64) as Float;
            }

            // perform edge tests
            if u < 0.0 || v < 0.0 || w < 0.0 {
                return None;
            }
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
            ShadingMode::Flat => (p1 - p0).cross(p2 - p0),
            ShadingMode::Phong => {
                let beta = u * inv_det;
                let gamma = v * inv_det;
                let alpha = 1.0 - beta - gamma;

                alpha * v0.normal + beta * v1.normal + gamma * v2.normal
            }
        }
        .normalized();

        Some(Intersection::new(point, normal, t, *ray))
    }

    #[cfg(not(feature = "watertight-mesh"))]
    fn intersect(&self, mesh: &Mesh, ray: &Ray) -> Option<Intersection> {
        let (v0, v1, v2) = self.get_vertices(&mesh.vertices);
        let (p0, p1, p2) = (v0.position, v1.position, v2.position);

        let edge1 = p1 - p0;
        let edge2 = p2 - p0;

        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        // ray is parallel to triangle
        if a.is_approx_zero() {
            return None;
        }

        let f = 1.0 / a;
        let s = ray.origin - p0;
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
                let alpha = 1.0 - beta - gamma;

                alpha * v0.normal + beta * v1.normal + gamma * v2.normal
            }
        }
        .normalized();

        Some(Intersection::new(point, normal, t, *ray))
    }

    #[cfg(feature = "watertight-mesh")]
    #[allow(clippy::many_single_char_names)]
    fn intersects(&self, vertices: &[Vertex], ray: &Ray) -> bool {
        let (v0, v1, v2) = self.get_vertices(vertices);
        let (p0, p1, p2) = (v0.position, v1.position, v2.position);

        let dir = ray.direction;
        // calculate dimension where the ray direction is maximal
        let kz = max_index(dir.abs());
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
        let a = p0 - ray.origin;
        let b = p1 - ray.origin;
        let c = p2 - ray.origin;

        // perform shear and scale of vertices
        let ax = a[kx] - sx * a[kz];
        let ay = a[ky] - sy * a[kz];
        let bx = b[kx] - sx * b[kz];
        let by = b[ky] - sy * b[kz];
        let cx = c[kx] - sx * c[kz];
        let cy = c[ky] - sy * c[kz];

        // calculate scaled barycentric coordinates
        #[allow(unused_mut)]
        let mut u = cx * by - cy * bx;
        #[allow(unused_mut)]
        let mut v = ax * cy - ay * cx;
        #[allow(unused_mut)]
        let mut w = bx * ay - by * ax;

        // perform edge tests
        if u < 0.0 || v < 0.0 || w < 0.0 {
            return false;
        }

        // fallback to test against edges using double precision
        #[cfg(not(feature = "f64"))]
        {
            if u == 0.0 {
                u = (cx as f64 * by as f64 - cy as f64 * bx as f64) as Float;
            }
            if v == 0.0 {
                v = (ax as f64 * cy as f64 - ay as f64 * cx as f64) as Float;
            }
            if w == 0.0 {
                w = (bx as f64 * ay as f64 - by as f64 * ax as f64) as Float;
            }

            // perform edge tests
            if u < 0.0 || v < 0.0 || w < 0.0 {
                return false;
            }
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
    fn intersects(&self, vertices: &[Vector3], ray: &Ray) -> bool {
        let (v0, v1, v2) = self.get_vertices(vertices);
        let (p0, p1, p2) = (v0.position, v1.position, v2.position);

        let edge1 = p1 - p0;
        let edge2 = p2 - p0;

        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        // ray is parallel to triangle
        if a.is_approx_zero() {
            return false;
        }

        let f = 1.0 / a;
        let s = ray.origin - p0;
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

#[derive(Clone, Deserialize, Serialize)]
pub struct FromObjFile {
    /// The path of the obj file
    path: String,
    /// Optional scaling (1st application)
    #[serde(default)]
    scale: Option<Vector3>,
    #[serde(default)]
    /// Optional rotation (2nd application)
    /// - params: (axis, angle)
    rotation: Option<(Vector3, Float)>,
    #[serde(default)]
    /// Optional translation (3rd application)
    translation: Option<Vector3>,
}

/// A mesh consists of vertices and triangles, allowing queries for intersections.
/// Depending on the [`MeshMode`](MeshMode), the intersection normals will be interpolated.
pub struct Mesh {
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
    bounds: Aabb,
    shading_mode: ShadingMode,
    bvh: Tree<Face>,
    obj: Option<FromObjFile>,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        faces: Vec<Face>,
        bounds: Aabb,
        shading_mode: ShadingMode,
        obj: Option<FromObjFile>,
    ) -> Self {
        Self {
            vertices,
            faces,
            bounds,
            shading_mode,
            bvh: Default::default(),
            obj,
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
        let obj_file = ObjFile::from(&path);

        let mut bounds = Aabb::empty();

        obj_file
            .vertices
            .iter()
            .for_each(|v| bounds = bounds.join_vec(v.position));

        Mesh::new(
            obj_file.vertices,
            obj_file.faces,
            bounds,
            shading_mode,
            Some(FromObjFile {
                path: path.as_ref().to_str().unwrap().into(),
                scale: None,
                translation: None,
                rotation: None,
            }),
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
    pub fn angle_weights(p0: Vector3, p1: Vector3, p2: Vector3) -> (Float, Float, Float) {
        debug_assert!(is_finite(&p0));
        debug_assert!(is_finite(&p1));
        debug_assert!(is_finite(&p2));

        let e01 = (p1 - p0).normalized();
        let e12 = (p2 - p1).normalized();
        let e20 = (p0 - p2).normalized();

        let w0 = e01.dot(-e20).fast_clamp(-1.0, 1.0);
        let w1 = e12.dot(-e01).fast_clamp(-1.0, 1.0);
        let w2 = e20.dot(-e12).fast_clamp(-1.0, 1.0);

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
    pub fn translate(&mut self, translation: Vector3) -> &mut Self {
        debug_assert!(is_finite(&translation));

        self.vertices
            .iter_mut()
            .for_each(|v| v.position += translation);
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
    pub fn scale(&mut self, scale: Vector3) -> &mut Self {
        debug_assert!(is_finite(&scale));

        for v in &mut self.vertices {
            v.position *= scale;
            v.normal /= scale;
            v.normal.normalize();
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
    pub fn rotate(&mut self, rotation: Rotation3) -> &mut Self {
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
    pub fn transform(&mut self, transformation: Matrix3) -> &mut Self {
        self.vertices.iter_mut().for_each(|v| {
            v.position = transformation * v.position;

            v.normal = transformation.inversed().transposed() * v.normal;
            v.normal.normalize();
        });
        self.update_bounds();

        self
    }

    pub fn update_bounds(&mut self) -> &mut Self {
        let mut new_bounds = Aabb::empty();
        self.vertices.iter_mut().for_each(|v| {
            new_bounds.min = new_bounds.min.min_by_component(v.position);
            new_bounds.max = new_bounds.max.max_by_component(v.position);
        });

        self.bounds = new_bounds;

        self
    }

    pub fn build_bvh(&mut self) -> &mut Self {
        self.bvh = Tree::new(self.faces.clone(), |f| f.bounds(&self.vertices));

        self
    }
}

impl Boundable for Mesh {
    #[inline]
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

        state.serialize_field("shading_mode", &self.shading_mode)?;

        if let Some(obj) = &self.obj {
            state.serialize_field("obj_path", obj)?;
        } else {
            state.serialize_field("vertices", &self.vertices)?;
            state.serialize_field("faces", &self.faces)?;
            state.serialize_field("bounds", &self.bounds)?;
            state.serialize_field("shading_mode", &self.shading_mode)?;
        }

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
            Faces,
            Bounds,
            ShadingMode,
            Obj,
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
                        formatter
                            .write_str("`vertices`, `faces`, `bounds`, `shading_mode` or `obj`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        match v {
                            "vertices" => Ok(Field::Vertices),
                            "faces" => Ok(Field::Faces),
                            "bounds" => Ok(Field::Bounds),
                            "shading_mode" => Ok(Field::ShadingMode),
                            "obj" => Ok(Field::Obj),
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
                let mut faces = None;
                let mut bounds = None;
                let mut shading_mode = None;
                let mut obj: Option<FromObjFile> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Vertices => {
                            if vertices.is_some() {
                                return Err(de::Error::duplicate_field("vertices"));
                            } else if obj.is_some() {
                                return Err(de::Error::custom("obj given with other options"));
                            } else {
                                vertices = Some(map.next_value()?);
                            }
                        }
                        Field::Faces => {
                            if faces.is_some() {
                                return Err(de::Error::duplicate_field("faces"));
                            } else if obj.is_some() {
                                return Err(de::Error::custom("obj given with other options"));
                            } else {
                                faces = Some(map.next_value()?);
                            }
                        }
                        Field::Bounds => {
                            if bounds.is_some() {
                                return Err(de::Error::duplicate_field("bounds"));
                            } else if obj.is_some() {
                                return Err(de::Error::custom("obj given with other options"));
                            } else {
                                bounds = Some(map.next_value()?);
                            }
                        }
                        Field::ShadingMode => {
                            if shading_mode.is_some() {
                                return Err(de::Error::duplicate_field("shading_mode"));
                            } else {
                                shading_mode = Some(map.next_value()?);
                            }
                        }
                        Field::Obj => {
                            if obj.is_some() {
                                return Err(de::Error::duplicate_field("obj"));
                            } else if vertices.is_some() || faces.is_some() || bounds.is_some() {
                                return Err(de::Error::custom("obj given with other options"));
                            } else {
                                obj = Some(map.next_value()?);
                            }
                        }
                    }
                }

                let shading_mode =
                    shading_mode.ok_or_else(|| de::Error::invalid_length(0, &self))?;

                if let Some(obj) = obj {
                    let mut mesh = Mesh::load(obj.path, shading_mode);

                    if let Some(scale) = obj.scale {
                        mesh.scale(scale);
                    }
                    if let Some((axis, angle)) = obj.rotation {
                        let rotation = Matrix3::from_rotation_around(axis, angle);
                        mesh.transform(rotation);
                    }
                    if let Some(translation) = obj.translation {
                        mesh.translate(translation);
                    }

                    return Ok(mesh);
                }

                let vertices = vertices.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let faces = faces.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let bounds = bounds.ok_or_else(|| de::Error::invalid_length(0, &self))?;

                Ok(Mesh::new(vertices, faces, bounds, shading_mode, None))
            }
        }

        const FIELDS: &[&str] = &["vertices", "faces", "bounds", "shading_mode", "obj"];
        deserializer
            .deserialize_struct("Mesh", FIELDS, MeshVisitor)
            .map(|mut m| {
                m.update_bounds().build_bvh();
                m
            })
    }
}
