use crate::bvh::Tree;
use crate::debug_util::{is_finite, is_normalized};
use crate::{Aabb, Boundable, Intersectable, Intersection, Ray};
use serde::{Deserialize, Serialize};
use std::mem::swap;
use tobj::Mesh as TobjMesh;
use ultraviolet::{Mat3, Rotor3, Vec3};
use utility::floats::fast_clamp;

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

    fn new_pos(position: Vec3) -> Self {
        debug_assert!(is_finite(&position));

        Self {
            position,
            normal: Vec3::zero(),
        }
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

    #[allow(clippy::many_single_char_names)]
    fn intersect(&self, mesh: &Mesh, ray: &Ray) -> Option<Intersection> {
        let vertex0 = mesh.vertices[self.a as usize];
        let vertex1 = mesh.vertices[self.b as usize];
        let vertex2 = mesh.vertices[self.c as usize];

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
        let a = vertex0.position - ray.origin;
        let b = vertex1.position - ray.origin;
        let c = vertex2.position - ray.origin;

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

        // let edge1 = vertex1.position - vertex0.position;
        // let edge2 = vertex2.position - vertex0.position;
        // let p_vec = ray.direction.cross(edge2);
        //
        // let det = edge1.dot(p_vec);
        // if det < EPSILON {
        //     return None;
        // }
        //
        // let t_vec = ray.origin - vertex0.position;
        //
        // let u = t_vec.dot(p_vec);
        // if u < 0.0 || u > det {
        //     return None;
        // }
        //
        // let q_vec = t_vec.cross(edge1);
        //
        // let v = ray.direction.dot(q_vec);
        // if v < 0.0 || u + v > det {
        //     return None;
        // }
        //
        // let inv_det = 1.0 / det;
        // let t = inv_det * edge2.dot(q_vec);
        // if !ray.contains(t) {
        //     return None;
        // }

        let point = ray.at(t);

        let normal = match mesh.shading_mode {
            ShadingMode::Flat => mesh.normals[self.normal as usize],
            ShadingMode::Phong => {
                let beta = u * inv_det;
                let gamma = v * inv_det;
                let alpha = 1.0 - beta - gamma;

                (alpha * vertex0.normal + beta * vertex1.normal + gamma * vertex2.normal)
                    .normalized()
            }
        };

        Some(Intersection::new(point, normal, t, *ray))
    }

    #[allow(clippy::many_single_char_names)]
    fn intersects(&self, mesh: &Mesh, ray: &Ray) -> bool {
        let vertex0 = mesh.vertices[self.a as usize];
        let vertex1 = mesh.vertices[self.b as usize];
        let vertex2 = mesh.vertices[self.c as usize];

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
        let a = vertex0.position - ray.origin;
        let b = vertex1.position - ray.origin;
        let c = vertex2.position - ray.origin;

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

        // let a = mesh.vertices[self.a as usize].position;
        // let b = mesh.vertices[self.b as usize].position;
        // let c = mesh.vertices[self.c as usize].position;
        //
        // let a_to_b = b - a;
        // let a_to_c = c - a;
        // let part0 = ray.direction.cross(a_to_c);
        //
        // let det = a_to_b.dot(part0);
        // if det < EPSILON {
        //     return false;
        // }
        //
        // let part1 = ray.origin - a;
        // let beta = part1.dot(part0) / det;
        // if !(0.0..=1.0).contains(&beta) {
        //     return false;
        // }
        //
        // let part2 = part1.cross(a_to_b);
        // let gamma = ray.direction.dot(part2) / det;
        // if gamma < 0.0 || beta + gamma > 1.0 {
        //     return false;
        // }
        //
        // let t = a_to_c.dot(part2) / det;
        // ray.contains(t)
    }
}

/// A mesh consists of vertices and triangles, allowing queries for intersections.
/// Depending on the [`MeshMode`](MeshMode), the intersection normals will be interpolated.
pub struct Mesh {
    vertices: Vec<Vertex>,
    normals: Vec<Vec3>,
    triangles: Vec<Triangle>,
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
        Self {
            vertices,
            normals,
            triangles,
            bounds,
            shading_mode,
            bvh: Tree::empty(),
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
    pub fn load(tobj_mesh: &TobjMesh, shading_mode: ShadingMode) -> Mesh {
        let mut bounds = Aabb::empty();

        let mut vertices = Vec::with_capacity(tobj_mesh.positions.len());
        let mut pos_index = 0;
        while pos_index < tobj_mesh.positions.len() {
            let position = Vec3::new(
                tobj_mesh.positions[pos_index],
                tobj_mesh.positions[pos_index + 1],
                tobj_mesh.positions[pos_index + 2],
            );

            bounds.min = bounds.min.min_by_component(position);
            bounds.max = bounds.max.max_by_component(position);

            let vertex = if tobj_mesh.normals.is_empty() {
                Vertex::new_pos(position)
            } else {
                let normal = Vec3::new(
                    tobj_mesh.normals[pos_index],
                    tobj_mesh.normals[pos_index + 1],
                    tobj_mesh.normals[pos_index + 2],
                );

                Vertex::new(position, normal)
            };

            vertices.push(vertex);

            pos_index += 3;
        }
        vertices.shrink_to_fit();
        assert_eq!(pos_index, tobj_mesh.positions.len());

        let mut normals = Vec::with_capacity(tobj_mesh.indices.len() / 3);
        let mut triangles = Vec::with_capacity(tobj_mesh.indices.len() / 3);

        let mut index = 0;
        while index < tobj_mesh.indices.len() {
            let a_index = tobj_mesh.indices[index];
            let b_index = tobj_mesh.indices[index + 1];
            let c_index = tobj_mesh.indices[index + 2];

            let p0 = vertices[a_index as usize].position;
            let p1 = vertices[b_index as usize].position;
            let p2 = vertices[c_index as usize].position;
            let normal = (p1 - p0).cross(p2 - p0).normalized();

            debug_assert_ne!(normal, Vec3::zero());

            let normal_index = normals.len() as u32;
            normals.push(normal);

            let triangle = Triangle::new(a_index, b_index, c_index, normal_index);
            triangles.push(triangle);

            index += 3;
        }
        triangles.shrink_to_fit();
        assert_eq!(index, tobj_mesh.indices.len());

        if tobj_mesh.normals.is_empty() {
            triangles.iter().for_each(|t| {
                let (w0, w1, w2) = Self::angle_weights(
                    vertices[t.a as usize].position,
                    vertices[t.b as usize].position,
                    vertices[t.c as usize].position,
                );
                debug_assert!(!(w0 == 0.0 && w1 == 0.0 && w2 == 0.0));

                let normal = normals[t.normal as usize];

                vertices[t.a as usize].normal += w0 * normal;
                vertices[t.b as usize].normal += w1 * normal;
                vertices[t.c as usize].normal += w2 * normal;
            });

            vertices.iter_mut().for_each(|v| {
                debug_assert!(is_finite(&v.normal));
                debug_assert_ne!(v.normal, Vec3::zero());
                v.normal.normalize()
            });
        }

        Mesh::new(vertices, normals, triangles, bounds, shading_mode)
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

        for v in &mut self.vertices {
            v.position *= scale;
            v.normal *= scale;
            v.normal.normalize();
        }

        // self.vertices.iter_mut().for_each(|v| {
        //     v.position *= scale;
        //     v.normal *= scale;
        //     v.normal.normalize();
        // });
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

    pub fn update_bounds(&mut self) {
        let mut new_bounds = Aabb::empty();
        self.vertices.iter_mut().for_each(|v| {
            new_bounds.min = new_bounds.min.min_by_component(v.position);
            new_bounds.max = new_bounds.min.max_by_component(v.position);
        });

        self.bounds = new_bounds;
    }

    pub fn build_bvh(&mut self) {
        self.bvh = Tree::new(self.triangles.clone(), |t| t.bounds(self));
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
        let hits = self.bvh.intersect(ray);
        if hits.is_empty() {
            return false;
        }

        hits.iter().any(|t| t.intersects(self, ray))
    }
}
