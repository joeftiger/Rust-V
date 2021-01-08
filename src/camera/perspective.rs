use crate::camera::Camera;
use crate::debug_utils::{is_finite, is_normalized, within_01};
use geometry::Ray;
use ultraviolet::{UVec2, Vec2, Vec3};
use utility::floats::in_range;

/// # Summary
/// A perspective camera with a fov somewhere in space, looking at a target.
pub struct PerspectiveCamera {
    position: Vec3,
    resolution: UVec2,
    x_dir: Vec3,
    y_dir: Vec3,
    lower_left: Vec3,
}

impl PerspectiveCamera {
    /// # Summary
    /// Creates a new perspective camera.
    ///
    /// # Constraints
    /// * `position` - All values should be finite (neither infinite nor `NaN`).
    /// * `target` - All values should be finite.
    /// * `up` - All values should be finite.
    ///          Should be normalized.
    /// * `fov_y` - Should be within `(0, 360)`.
    ///
    /// # Arguments
    /// * `position` - The eye / position of the camera
    /// * `target` - Where the camera looks at
    /// * `up` - The up vector of the camera
    /// * `fov_y` - The field of view (in degrees)
    /// * `resolution` - The resolution of the camera
    ///
    /// # Returns
    /// * Self
    pub fn new(position: Vec3, target: Vec3, up: Vec3, fov_y: f32, resolution: UVec2) -> Self {
        debug_assert!(is_finite(&position));
        debug_assert!(is_finite(&target));
        debug_assert!(is_finite(&up));
        debug_assert!(is_normalized(&up));
        debug_assert!(in_range(fov_y, 0.0, 360.0));

        // compute orientation and distance of eye to scene center
        let view = (target - position).normalized();
        let distance = (target - position).mag();

        let w = resolution.x as f32;
        let h = resolution.y as f32;
        let image_height = 2.0 * distance * (0.5 * fov_y).to_radians().tan();
        let image_width = w / h * image_height;

        // compute width & height of the image plane
        // based on the opening angle of the camera (fov_y) and the distance
        // of the eye to the near plane (distance)
        let x_dir = view.cross(up).normalized() * image_width / w;
        let y_dir = -up * image_height / h;

        let lower_left = target - 0.5 * w * x_dir - 0.5 * h * y_dir;

        Self {
            position,
            resolution,
            x_dir,
            y_dir,
            lower_left,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn primary_ray(&self, pixel: &UVec2, sample: &Vec2) -> Ray {
        debug_assert!(*pixel == pixel.min_by_component(self.resolution));
        debug_assert!(within_01(sample));

        let direction = self.lower_left
            + (pixel.x as f32 + sample.x) * self.x_dir
            + (pixel.y as f32 + sample.y) * self.y_dir
            - self.position;

        Ray::new_fast(self.position, direction.normalized())
    }
}
