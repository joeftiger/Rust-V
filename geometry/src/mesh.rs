use crate::debug_util::{is_finite, is_normalized};
use crate::{Aabb, Boundable, Cube, Intersectable, Intersection, Ray};
use std::sync::Arc;
use tobj::Mesh as TobjMesh;
use ultraviolet::{Mat3, Rotor3, Vec3};
use utility::floats::EPSILON;

/// The shading mode defines the shading of normals. In `Flat` mode, the surface of triangles will
/// appear flat. In `Phong` however, they will be interpolated to create a smooth looking surface.
pub enum ShadingMode {
    Flat,
    Phong,
}

/// A vertex consists of a position and normal vector, which possibly influences the shading of
/// [`triangles`](Triangle).
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
    a: Vec3,
    b: Vec3,
    c: Vec3,
    n_a: Vec3,
    n_b: Vec3,
    n_c: Vec3,
    normal: Vec3,
}

impl Triangle {
    /// Creates a new triangle.
    ///
    /// # Constraints
    /// * `normal` - All values should be finite (neither infinite nor `NaN`).
    ///              Should be normalized.
    ///
    /// # Arguments
    /// * `a` - The vertex index of a mesh
    /// * `b` - The vertex index of a mesh
    /// * `c` - The vertex index of a mesh
    /// * `normal` - The triangle normal vector
    ///
    /// # Returns
    /// * Self
    fn new(a: Vec3, b: Vec3, c: Vec3, n_a: Vec3, n_b: Vec3, n_c: Vec3, normal: Vec3) -> Self {
        debug_assert!(is_finite(&a));
        debug_assert!(is_finite(&b));
        debug_assert!(is_finite(&c));
        debug_assert!(is_finite(&normal));
        debug_assert!(is_normalized(&normal));

        Self {
            a,
            b,
            c,
            n_a,
            n_b,
            n_c,
            normal,
        }
    }
}

impl Boundable for Triangle {
    fn bounds(&self) -> Cube {
        let min = self.a.min_by_component(self.b.min_by_component(self.c));
        let max = self.a.max_by_component(self.b.max_by_component(self.c));

        Cube::new(min, max)
    }
}

impl Intersectable for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let a1 = -ray.direction;
        let a2 = self.b - self.a;
        let a3 = self.c - self.a;

        let denom = Mat3::new(a1, a2, a3).determinant();
        if denom.abs() < EPSILON {
            return None;
        }

        let b = ray.origin - self.a;

        let beta = Mat3::new(a1, b, a3).determinant() / denom;
        if !(0.0..=1.0).contains(&beta) {
            return None;
        }

        let gamma = Mat3::new(a1, a2, b).determinant() / denom;
        if gamma < 0.0 || beta + gamma > 1.0 {
            return None;
        }

        let t = Mat3::new(b, a2, a3).determinant() / denom;

        // let a_to_b = self.b - self.a;
        // let a_to_c = self.c - self.a;
        // let part0 = ray.direction.cross(a_to_c);
        //
        // let det = a_to_b.dot(part0);
        // if det < EPSILON {
        //     return None;
        // }
        //
        // let part1 = ray.origin - self.a;
        // let beta = part1.dot(part0) / det;
        // if !(0.0..=1.0).contains(&beta) {
        //     return None;
        // }
        //
        // let part2 = part1.cross(a_to_b);
        // let gamma = ray.direction.dot(part2) / det;
        // if gamma < 0.0 || beta + gamma > 1.0 {
        //     return None;
        // }
        //
        // let t = a_to_c.dot(part2) / det;
        if !ray.contains(t) {
            return None;
        }

        let point = ray.at(t);
        let normal = match self.mesh.mode {
            ShadingMode::Flat => self.normal,
            ShadingMode::Phong => {
                let alpha = 1.0 - beta - gamma;

                let n0 = vertex_a.normal;
                let n1 = vertex_b.normal;
                let n2 = vertex_c.normal;

                (alpha * n0 + beta * n1 + gamma * n2).normalized()
            }
        };

        Some(Intersection::new(point, normal, t, *ray))
    }

    fn intersects(&self, ray: &Ray) -> bool {
        let vertex_a = &self.mesh.vertices[self.a as usize];
        let vertex_b = &self.mesh.vertices[self.b as usize];
        let vertex_c = &self.mesh.vertices[self.c as usize];

        let a = vertex_a.position;
        let b = vertex_b.position;
        let c = vertex_c.position;

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
    triangles: Vec<Triangle>,
    mode: ShadingMode,
    bounds: Aabb,
}

impl Mesh {
    fn new(
        vertices: Vec<Vertex>,
        triangles: Vec<Triangle>,
        mode: ShadingMode,
        bounds: Aabb,
    ) -> Self {
        Self {
            vertices,
            triangles,
            mode,
            bounds,
        }
    }

    /// Loads the given tobj mesh.
    ///
    /// # Arguments
    /// * `tobj_mesh` - The tobj mesh to load
    /// * `mode` - The shading mode
    ///
    /// # Returns
    /// * Self
    pub fn load(tobj_mesh: &TobjMesh, mode: ShadingMode) -> Self {
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

        let mut mesh = Self {
            vertices,
            triangles: Vec::with_capacity(tobj_mesh.indices.len() / 3),
            mode,
            bounds,
        };

        let m = Arc::new(mesh);

        let mut j = 0;
        while j < tobj_mesh.indices.len() {
            let a = tobj_mesh.indices[j];
            let b = tobj_mesh.indices[j + 1];
            let c = tobj_mesh.indices[j + 2];

            let p0 = m.vertices[a as usize].position;
            let p1 = m.vertices[b as usize].position;
            let p2 = m.vertices[c as usize].position;
            let normal = (p1 - p0).cross(p2 - p0).normalized();

            let triangle = Triangle::new(mesh, a, b, c, normal);
            m.triangles.push(triangle);

            j += 3;
        }
        m.triangles.shrink_to_fit();
        assert_eq!(j, tobj_mesh.indices.len());

        *m
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
        let mut new_bounds = Aabb::empty();
        self.vertices.iter_mut().for_each(|v| {
            v.position = rotation * v.position;

            new_bounds.min = new_bounds.min.min_by_component(v.position);
            new_bounds.max = new_bounds.min.max_by_component(v.position);
        });

        self.bounds = new_bounds;

        self
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

    pub fn update_bounds(&mut self) {
        let mut new_bounds = Aabb::empty();
        self.vertices.iter_mut().for_each(|v| {
            new_bounds.min = new_bounds.min.min_by_component(v.position);
            new_bounds.max = new_bounds.min.max_by_component(v.position);
        });

        self.bounds = new_bounds;
    }
}

impl Boundable for Mesh {
    fn bounds(&self) -> Cube {
        self.bounds.into()
    }
}

impl Intersectable for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        if self.bounds.intersects(ray) {
            return None;
        }

        let mut new_ray = *ray;
        let mut intersection = None;

        for triangle in &self.triangles {
            if let Some(i) = triangle.intersect(&new_ray) {
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
        self.bounds.intersects(ray)
            && self
                .triangles
                .iter()
                .any(|triangle| triangle.intersects(ray))
    }
}
