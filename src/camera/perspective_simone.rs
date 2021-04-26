use crate::camera::Camera;
use crate::samplers::camera::CameraSampler;
use definitions::{Float, Matrix4, Vector2, Vector3};
use geometry::Ray;
use serde::{Deserialize, Serialize};
use ultraviolet::UVec2;

#[derive(Serialize, Deserialize)]
pub struct PerspectiveCameraSimone {
    resolution: UVec2,
    sampler: CameraSampler,
    look_at: Matrix4,
    bottom_left: Vector2,
    top_right: Vector2,
    inv_resolution: Vector2,
}

impl PerspectiveCameraSimone {
    pub fn new(
        sampler: CameraSampler,
        position: Vector3,
        target: Vector3,
        up: Vector3,
        fov: Float,
        resolution: UVec2,
    ) -> Self {
        let look_at = look_at(position, target, up);

        let y = (0.5 * fov).to_radians().tan();
        let x = y * resolution.x as Float / resolution.y as Float;

        let top_right = Vector2::new(x, y);
        let bottom_left = -top_right;

        let inv_resolution = Vector2::new(1.0 / resolution.x as Float, 1.0 / resolution.y as Float);

        Self {
            resolution,
            sampler,
            look_at,
            bottom_left,
            top_right,
            inv_resolution,
        }
    }
}

#[typetag::serde]
impl Camera for PerspectiveCameraSimone {
    fn resolution(&self) -> UVec2 {
        self.resolution
    }

    fn primary_ray(&self, pixel: UVec2) -> Ray {
        let dir_2d = self.bottom_left
            + (self.top_right - self.bottom_left)
                * (to_vec2(pixel) * self.sampler.sample())
                * self.inv_resolution;
        let dir_3d = Vector3::new(dir_2d.x, dir_2d.y, -1.0);

        let origin = self.look_at.transform_vec3(Vector3::zero());
        let direction = self.look_at.transform_vec3(dir_3d).normalized();

        Ray::new_fast(origin, direction)
    }
}

fn look_at(position: Vector3, target: Vector3, up: Vector3) -> Matrix4 {
    let w = (position - target).normalized();
    let u = up.cross(w).normalized();
    let v = w.cross(u);

    Matrix4::new(
        u.into_homogeneous_vector(),
        v.into_homogeneous_vector(),
        w.into_homogeneous_vector(),
        position.into_homogeneous_point(),
    )
}

fn to_vec2(v: UVec2) -> Vector2 {
    Vector2::new(v.x as Float, v.y as Float)
}
