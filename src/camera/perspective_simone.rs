use crate::camera::Camera;
use geometry::Ray;
use ultraviolet::{Mat4, UVec2, Vec2, Vec3};

pub struct PerspectiveCamera {
    look_at: Mat4,
    bottom_left: Vec2,
    top_right: Vec2,
    inv_resolution: Vec2,
}

impl PerspectiveCamera {
    pub fn new(position: Vec3, target: Vec3, up: Vec3, fov: f32, resolution: UVec2) -> Self {
        let look_at = look_at(position, target, up);

        let y = (0.5 * fov).to_radians().tan();
        let x = y * resolution.x as f32 / resolution.y as f32;

        let top_right = Vec2::new(x, y);
        let bottom_left = -top_right;

        let inv_resolution = Vec2::new(1.0 / resolution.x as f32, 1.0 / resolution.y as f32);

        Self {
            look_at,
            bottom_left,
            top_right,
            inv_resolution,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn primary_ray(&self, pixel: &UVec2, sample: &Vec2) -> Ray {
        let dir_2d = self.bottom_left
            + (self.top_right - self.bottom_left)
                * (to_vec2(pixel) * *sample)
                * self.inv_resolution;
        let dir_3d = Vec3::new(dir_2d.x, dir_2d.y, -1.0);

        let origin = self.look_at.transform_vec3(Vec3::zero());
        let direction = self.look_at.transform_vec3(dir_3d).normalized();

        Ray::new_fast(origin, direction)
    }
}

fn look_at(position: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let w = (position - target).normalized();
    let u = up.cross(w).normalized();
    let v = w.cross(u);

    Mat4::new(
        u.into_homogeneous_vector(),
        v.into_homogeneous_vector(),
        w.into_homogeneous_vector(),
        position.into_homogeneous_point(),
    )
}

fn to_vec2(v: &UVec2) -> Vec2 {
    Vec2::new(v.x as f32, v.y as f32)
}
