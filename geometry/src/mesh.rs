use crate::debug_util::{is_finite, is_normalized};
use crate::{Boundable, Cube, Intersectable, Intersection, Ray};
use ultraviolet::{Mat3, Rotor3, Vec3};
use utility::floats::EPSILON;

/// The mesh mode defines the shading of normals. In `Flat` mode, the surface of triangles will
/// appear flat. In `Phong` however, they will be interpolated to create a smooth looking surface.
pub enum MeshMode {
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
#[derive(Copy, Clone)]
struct Triangle {
    a: u32,
    b: u32,
    c: u32,
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
    fn new(a: u32, b: u32, c: u32, normal: Vec3) -> Self {
        debug_assert!(is_finite(&normal));
        debug_assert!(is_normalized(&normal));

        Self { a, b, c, normal }
    }

    /// Returns the bounds of this triangle with vertices of the given mesh.
    ///
    /// # Constraints
    /// * `mesh` - Should really contain this triangle's vertices.
    ///
    /// # Arguments
    /// * `mesh` - The mesh to retrieve the vertices from
    ///
    /// # Returns
    /// * The bounds of this triangle
    fn bounds(&self, mesh: &Mesh) -> Cube {
        debug_assert!(self.a < mesh.vertices.len() as u32);
        debug_assert!(self.b < mesh.vertices.len() as u32);
        debug_assert!(self.c < mesh.vertices.len() as u32);

        let a = mesh.vertices[self.a as usize].position;
        let b = mesh.vertices[self.b as usize].position;
        let c = mesh.vertices[self.c as usize].position;

        let min = a.min_by_component(b.min_by_component(c));
        let max = a.max_by_component(b.max_by_component(c));

        Cube::new(min, max)
    }

    /// Intersects this triangle of the given mesh with a ray.
    /// Depending on the [`MeshMode`](MeshMode) of the mesh, the surface normal - if any - will be
    /// interpolated.
    ///
    /// # Constraints
    /// * `mesh` - Should really contain this triangle's vertices.
    ///
    /// # Arguments
    /// * `mesh` - The mesh to retrieve the vertices from
    /// * `ray` - The ray to intersect
    ///
    /// # Returns
    /// Either
    /// * `Some` intersection, or
    /// * `None`
    fn intersect(&self, mesh: &Mesh, ray: &Ray) -> Option<Intersection> {
        debug_assert!(self.a < mesh.vertices.len() as u32);
        debug_assert!(self.b < mesh.vertices.len() as u32);
        debug_assert!(self.c < mesh.vertices.len() as u32);

        let vertex_a = &mesh.vertices[self.a as usize];
        let vertex_b = &mesh.vertices[self.b as usize];
        let vertex_c = &mesh.vertices[self.c as usize];

        let a = vertex_a.position;
        let b = vertex_b.position;
        let c = vertex_c.position;

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
        let normal = match mesh.mode {
            MeshMode::Flat => self.normal,
            MeshMode::Phong => {
                let alpha = 1.0 - beta - gamma;

                let n0 = vertex_a.normal;
                let n1 = vertex_b.normal;
                let n2 = vertex_c.normal;

                (alpha * n0 + beta * n1 + gamma * n2).normalized()
            }
        };

        Some(Intersection::new(point, normal, t, *ray))
    }

    /// Checks for an intersection of this triangle of the given mesh with a ray.
    ///
    /// # Constraints
    /// * `mesh` - Should really contain this triangle's vertices.
    ///
    /// # Arguments
    /// * `mesh` - The mesh to retrieve the vertices from
    /// * `ray` - The ray to intersect
    ///
    /// # Returns
    /// * Whether an intersection occurs
    fn intersects(&self, mesh: &Mesh, ray: &Ray) -> bool {
        debug_assert!(self.a < mesh.vertices.len() as u32);
        debug_assert!(self.b < mesh.vertices.len() as u32);
        debug_assert!(self.c < mesh.vertices.len() as u32);

        let vertex_a = &mesh.vertices[self.a as usize];
        let vertex_b = &mesh.vertices[self.b as usize];
        let vertex_c = &mesh.vertices[self.c as usize];

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
    mode: MeshMode,
    bounds: Cube,
}

impl Mesh {
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
        let mut new_bounds = Cube::empty();
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
    fn bounds(&self) -> Cube {
        self.bounds
    }
}

impl Intersectable for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let mut new_ray = *ray;
        let mut intersection = None;

        for triangle in &self.triangles {
            if let Some(i) = triangle.intersect(self, &new_ray) {
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
        self.triangles
            .iter()
            .any(|triangle| triangle.intersects(self, ray))
    }
}
