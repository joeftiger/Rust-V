use crate::camera::Camera;
use crate::debug_utils::{is_finite, is_normalized};
use crate::sampler::pixel_samplers::{PixelSampler, PixelSamplerType};
use definitions::{Float, Vector3};
use geometry::Ray;
use serde::de::{Error, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use ultraviolet::UVec2;
use utility::floats::FloatExt;

/// A perspective camera with a fov somewhere in space, looking at a target.
pub struct PerspectiveCamera {
    sampler: PixelSamplerType,
    position: Vector3,
    target: Vector3,
    up: Vector3,
    fovy: Float,
    resolution: UVec2,
    x_dir: Vector3,
    y_dir: Vector3,
    lower_left: Vector3,
}

impl PerspectiveCamera {
    /// Creates a new perspective camera.
    ///
    /// # Constraints
    /// * `position` - All values should be finite (neither infinite nor `NaN`).
    /// * `target` - All values should be finite.
    /// * `up` - All values should be finite.
    ///          Should be normalized.
    /// * `fovy` - Should be within `(0, 360)`.
    ///
    /// # Arguments
    /// * `position` - The eye / position of the camera
    /// * `target` - Where the camera looks at
    /// * `up` - The up vector of the camera
    /// * `fovy` - The field of view (in degrees)
    /// * `resolution` - The resolution of the camera
    ///
    /// # Returns
    /// * Self
    pub fn new(
        sampler: PixelSamplerType,
        position: Vector3,
        target: Vector3,
        up: Vector3,
        fovy: Float,
        resolution: UVec2,
    ) -> Self {
        debug_assert!(is_finite(&position));
        debug_assert!(is_finite(&target));
        debug_assert!(is_finite(&up));
        debug_assert!(is_normalized(&up));
        debug_assert!(fovy.in_range(0.0, 360.0));

        // compute orientation and distance of eye to scene center
        let view = (target - position).normalized();
        let distance = (target - position).mag();

        let w = resolution.x as Float;
        let h = resolution.y as Float;
        let image_height = 2.0 * distance * (0.5 * fovy).to_radians().tan();
        let image_width = w / h * image_height;

        // compute width & height of the image plane
        // based on the opening angle of the camera (fovy) and the distance
        // of the eye to the near plane (distance)
        let x_dir = view.cross(up).normalized() * image_width / w;
        let y_dir = -up * image_height / h;

        let lower_left = target - 0.5 * w * x_dir - 0.5 * h * y_dir;

        Self {
            sampler,
            position,
            target,
            up,
            fovy,
            resolution,
            x_dir,
            y_dir,
            lower_left,
        }
    }
}

#[typetag::serde]
impl Camera for PerspectiveCamera {
    fn resolution(&self) -> UVec2 {
        self.resolution
    }

    fn primary_ray(&self, pixel: UVec2) -> Ray {
        debug_assert!(pixel == pixel.min_by_component(self.resolution));

        let sample = self.sampler.sample(pixel);

        let direction = self.lower_left
            + (pixel.x as Float + sample.x) * self.x_dir
            + (pixel.y as Float + sample.y) * self.y_dir
            - self.position;

        Ray::new_fast(self.position, direction.normalized())
    }
}

impl Serialize for PerspectiveCamera {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PerspectiveCamera", 6)?;
        state.serialize_field("Sampler", &self.sampler)?;
        state.serialize_field("Position", &self.position)?;
        state.serialize_field("Target", &self.target)?;
        state.serialize_field("Up", &self.up)?;
        state.serialize_field("FovY", &self.fovy)?;
        state.serialize_field("Resolution", &self.resolution)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for PerspectiveCamera {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Sampler,
            Position,
            Target,
            Up,
            FovY,
            Resolution,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(
                            "`Sampler`, `Position`, `Target`, `Up`, `FovY` or `Resolution`",
                        )
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        match v {
                            "Sampler" => Ok(Field::Sampler),
                            "Position" => Ok(Field::Position),
                            "Target" => Ok(Field::Target),
                            "Up" => Ok(Field::Up),
                            "FovY" => Ok(Field::FovY),
                            "Resolution" => Ok(Field::Resolution),
                            _ => Err(de::Error::unknown_field(v, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct CameraVisitor;

        impl<'de> Visitor<'de> for CameraVisitor {
            type Value = PerspectiveCamera;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct PerspectiveCamera")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
            where
                A: MapAccess<'de>,
            {
                let mut sampler = None;
                let mut position = None;
                let mut target = None;
                let mut up = None;
                let mut fovy = None;
                let mut resolution = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Sampler => {
                            if sampler.is_some() {
                                return Err(de::Error::duplicate_field("Sampler"));
                            } else {
                                sampler = Some(map.next_value()?)
                            }
                        }
                        Field::Position => {
                            if position.is_some() {
                                return Err(de::Error::duplicate_field("Position"));
                            } else {
                                position = Some(map.next_value()?)
                            }
                        }
                        Field::Target => {
                            if target.is_some() {
                                return Err(de::Error::duplicate_field("Target"));
                            } else {
                                target = Some(map.next_value()?)
                            }
                        }
                        Field::Up => {
                            if up.is_some() {
                                return Err(de::Error::duplicate_field("Up"));
                            } else {
                                up = Some(map.next_value()?)
                            }
                        }
                        Field::FovY => {
                            if fovy.is_some() {
                                return Err(de::Error::duplicate_field("FovY"));
                            } else {
                                fovy = Some(map.next_value()?)
                            }
                        }
                        Field::Resolution => {
                            if resolution.is_some() {
                                return Err(de::Error::duplicate_field("Resolution"));
                            } else {
                                resolution = Some(map.next_value()?)
                            }
                        }
                    }
                }

                let sampler = sampler.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let position = position.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let target = target.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let up = up.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let fovy = fovy.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let resolution = resolution.ok_or_else(|| de::Error::invalid_length(0, &self))?;

                Ok(PerspectiveCamera::new(
                    sampler, position, target, up, fovy, resolution,
                ))
            }
        }

        const FIELDS: &[&str] = &["Sampler", "Position", "Target", "Up", "FovY", "Resolution"];
        deserializer.deserialize_struct("PerspectiveCamera", FIELDS, CameraVisitor)
    }
}
