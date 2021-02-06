use crate::bvh::Tree;
use crate::debug_util::{is_finite, is_normalized};
use crate::{Aabb, Boundable, Container, Intersectable, Intersection, Ray};
use tobj::Mesh as TobjMesh;
use ultraviolet::{Mat3, Rotor3, Vec3};
use utility::floats::EPSILON;

/// The shading mode defines the shading of normals. In `Flat` mode, the surface of triangles will
/// appear flat. In `Phong` however, they will be interpolated to create a smooth looking surface.
#[derive(Debug)]
pub enum ShadingMode {
    Flat,
    Phong,
}

/// A vertex consists of a position and normal vector, which possibly influences the shading of
/// [`triangles`](Triangle).
#[derive(Copy, Clone)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
}

impl Vertex {
    /// Creates a new vertex.
    ///
    /// # Constraints
    /// * `position` - All values should be finite (neither infinite nor `NaN`).
    /// * `normal` - ALl values should be finite.
    ///              Should be normalized.
    ///
    /// # Arguments
    /// * `position` - The position of the vertex
    /// * `normal` - The normal of the vertex
    ///
    /// # Returns
    /// * Self
    fn new(position: Vec3, normal: Vec3) -> Self {
        debug_assert!(is_finite(&position));
        debug_assert!(is_finite(&normal));
        debug_assert!(is_normalized(&normal));

        Self { position, normal }
    }
}

/// A triangle consists of vertex indices `{a, b, c}` and a flat surface normal.
///
/// In order to query a triangle for an intersection, it is therefore needed to pass it the proper
/// [`Mesh`](Mesh) it resides in.
#[derive(Clone)]
struct Triangle {
    a: u32,
    b: u32,
    c: u32,
    normal: u32,
}

impl Triangle {
    /// Creates a new triangle.
    ///
    /// # Constraints
    /// * `normal` - All values should be finite (neither infinite nor `NaN`).
    ///              Should be normalized.
    ///
    /// # Arguments
    /// * `v0` - The vertex 0
    /// * `v1` - The vertex 1
    /// * `v2` - The vertex 2
    /// * `normal` - The triangle normal vector
    ///
    /// # Returns
    /// * Self
    fn new(a: u32, b: u32, c: u32, normal: u32) -> Self {
        Self { a, b, c, normal }
    }

    fn bounds(&self, mesh: &Mesh) -> Aabb {
        let a = mesh.vertices[self.a as usize].position;
        let b = mesh.vertices[self.b as usize].position;
        let c = mesh.vertices[self.c as usize].position;

        let min = a.min_by_component(b.min_by_component(c));
        let max = a.max_by_component(b.max_by_component(c));

        Aabb::new(min, max)
    }

    fn intersect(&self, mesh: &Mesh, ray: &Ray) -> Option<Intersection> {
        let v0 = mesh.vertices[self.a as usize];
        let v1 = mesh.vertices[self.b as usize];
        let v2 = mesh.vertices[self.c as usize];

        let a = v0.position;
        let b = v1.position;
        let c = v2.position;

        // let a1 = -ray.direction;
        // let a2 = b - a;
        // let a3 = c - a;
        //
        // let denom = Mat3::new(a1, a2, a3).determinant();
        // if denom.abs() < EPSILON {
        //     return None;
        // }
        //
        // let b = ray.origin - a;
        //
        // let beta = Mat3::new(a1, b, a3).determinant() / denom;
        // if !(0.0..=1.0).contains(&beta) {
        //     return None;
        // }
        //
        // let gamma = Mat3::new(a1, a2, b).determinant() / denom;
        // if gamma < 0.0 || beta + gamma > 1.0 {
        //     return None;
        // }
        //
        // let t = Mat3::new(b, a2, a3).determinant() / denom;

        let a_to_b = b - a;
        let a_to_c = c - a;
        let part0 = ray.direction.cross(a_to_c);

        let det = a_to_b.dot(part0);
        if det < EPSILON {
            return None;
        }

        let part1 = ray.origin - a;
        let beta = part1.dot(part0) / det;
        if !(0.0..=1.0).contains(&beta) {
            return None;
        }

        let part2 = part1.cross(a_to_b);
        let gamma = ray.direction.dot(part2) / det;
        if gamma < 0.0 || beta + gamma > 1.0 {
            return None;
        }

        let t = a_to_c.dot(part2) / det;
        if !ray.contains(t) {
            return None;
        }

        let point = ray.at(t);
        let normal = match mesh.shading_mode {
            ShadingMode::Flat => mesh.normals[self.normal as usize],
            ShadingMode::Phong => {
                let alpha = 1.0 - beta - gamma;

                let n0 = v0.normal;
                let n1 = v1.normal;
                let n2 = v2.normal;

                (alpha * n0 + beta * n1 + gamma * n2).normalized()
            }
        };

        Some(Intersection::new(point, normal, t, *ray))
    }

    fn intersects(&self, mesh: &Mesh, ray: &Ray) -> bool {
        let a = mesh.vertices[self.a as usize].position;
        let b = mesh.vertices[self.b as usize].position;
        let c = mesh.vertices[self.c as usize].position;

        let a_to_b = b - a;
        let a_to_c = c - a;
        let part0 = ray.direction.cross(a_to_c);

        let det = a_to_b.dot(part0);
        if det < EPSILON {
            return false;
        }

        let part1 = ray.origin - a;
        let beta = part1.dot(part0) / det;
        if !(0.0..=1.0).contains(&beta) {
            return false;
        }

        let part2 = part1.cross(a_to_b);
        let gamma = ray.direction.dot(part2) / det;
        if gamma < 0.0 || beta + gamma > 1.0 {
            return false;
        }

        let t = a_to_c.dot(part2) / det;
        ray.contains(t)
    }
}

/// A mesh consists of vertices and triangles, allowing queries for intersections.
/// Depending on the [`MeshMode`](MeshMode), the intersection normals will be interpolated.
pub struct Mesh {
    vertices: Vec<Vertex>,
    normals: Vec<Vec3>,
    bounds: Aabb,
    shading_mode: ShadingMode,
    bvh: Tree<Triangle>,
}

impl Mesh {
    fn new(
        vertices: Vec<Vertex>,
        normals: Vec<Vec3>,
        triangles: Vec<Triangle>,
        bounds: Aabb,
        shading_mode: ShadingMode,
    ) -> Self {
        let mut this = Self {
            vertices,
            normals,
            bounds,
            shading_mode,
            bvh: Tree::empty(),
        };

        this.bvh = Tree::new(triangles, |t| t.bounds(&this));

        this
    }

    /// Loads the given tobj mesh.
    ///
    /// # Arguments
    /// * `tobj_mesh` - The tobj mesh to load
    /// * `shading_mode` - The shading mode
    ///
    /// # Returns
    /// * Self
    pub fn load(tobj_mesh: &TobjMesh, shading_mode: ShadingMode) -> Mesh {
        assert!(!tobj_mesh.normals.is_empty());
        assert_eq!(tobj_mesh.positions.len(), tobj_mesh.normals.len());

        let mut bounds = Aabb::empty();

        let mut vertices = Vec::with_capacity(tobj_mesh.positions.len());
        let mut i = 0;
        while i < tobj_mesh.positions.len() {
            let position = Vec3::new(
                tobj_mesh.positions[i],
                tobj_mesh.positions[i + 1],
                tobj_mesh.positions[i + 2],
            );
            let normal = Vec3::new(
                tobj_mesh.normals[i],
                tobj_mesh.normals[i + 1],
                tobj_mesh.normals[i + 2],
            );

            bounds.min = bounds.min.min_by_component(position);
            bounds.max = bounds.max.max_by_component(position);

            let vertex = Vertex::new(position, normal);
            vertices.push(vertex);

            i += 3;
        }
        vertices.shrink_to_fit();
        assert_eq!(i, tobj_mesh.positions.len());

        let mut normals = Vec::with_capacity(tobj_mesh.indices.len() / 3);
        let mut triangles = Vec::with_capacity(tobj_mesh.indices.len() / 3);

        let mut j = 0;
        while j < tobj_mesh.indices.len() {
            let a = tobj_mesh.indices[j];
            let b = tobj_mesh.indices[j + 1];
            let c = tobj_mesh.indices[j + 2];

            let p0 = vertices[a as usize].position;
            let p1 = vertices[b as usize].position;
            let p2 = vertices[c as usize].position;
            let normal = (p1 - p0).cross(p2 - p0).normalized();

            let n = normals.len() as u32;
            normals.push(normal);

            let triangle = Triangle::new(a, b, c, n);
            triangles.push(triangle);

            j += 3;
        }
        triangles.shrink_to_fit();
        assert_eq!(j, tobj_mesh.indices.len());

        Mesh::new(vertices, normals, triangles, bounds, shading_mode)
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
    pub fn scale(&mut self, scale: Vec3) -> &mut Self {
        debug_assert!(is_finite(&scale));

        self.vertices.iter_mut().for_each(|v| v.position *= scale);
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
            v.position = transformation * v.position;

            new_bounds.min = new_bounds.min.min_by_component(v.position);
            new_bounds.max = new_bounds.min.max_by_component(v.position);
        });

        self.bounds = new_bounds;

        self
    }
}

impl Boundable for Mesh {
    fn bounds(&self) -> Aabb {
        self.bounds.into()
    }
}

impl Intersectable for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let mut new_ray = *ray;
        let mut intersection = None;

        let hits = self.bvh.intersect(ray);
        for hit in &hits {
            if let Some(i) = hit.intersect(self, &new_ray) {
                println!("not empty");
                new_ray.t_end = i.t;
                intersection = Some(i);
            }
        }

        if let Some(mut i) = intersection {
            println!("REALLY not empty");
            i.ray = *ray;
            Some(i)
        } else {
            None
        }
    }

    fn intersects(&self, ray: &Ray) -> bool {
        !self.bvh.intersect(ray).is_empty()
    }
}
