use crate::bxdf::BSDF;

use crate::debug_utils::{is_finite, is_normalized, within_01};
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::{Color, IndexSpectral};
use definitions::{Float, Vector2, Vector3};
use geometry::{Aabb, Boundable, Geometry, Intersectable, Intersection, Ray};
use serde::{Deserialize, Serialize};
use utility::floats::FloatExt;

/// An emitter is similar to a receiver, consisting of a geometry and a BSDF. Additionally, the
/// emitter also has an emission.
#[derive(Serialize, Deserialize)]
pub struct Emitter {
    geometry: Box<dyn Sampleable>,
    pub bsdf: BSDF,
    pub emission: Spectrum,
}

impl Emitter {
    /// Creates a new emitter.
    ///
    /// # Arguments
    /// * `geometry` - The geometry of the emitter
    /// * `bsdf` - The BSDF of the emitter
    /// * `emission` - The emission of the emitter
    ///
    /// # Returns
    /// * Self
    pub fn new(geometry: Box<dyn Sampleable>, bsdf: BSDF, emission: Spectrum) -> Self {
        Self {
            geometry,
            bsdf,
            emission,
        }
    }

    pub fn emission_light_wave(&self, light_wave_index: usize) -> Float {
        debug_assert!(light_wave_index < Spectrum::size());

        self.emission.index_spectral(light_wave_index)
    }

    /// Returns the radiance of this emitter, comparing the incident and normal vector.
    ///
    /// # Constraints
    /// * `incident` - All values should be finite (neither infinite nor `NaN`).
    ///                Should be normalized.
    /// * `normal` - All values should be finite.
    ///              Should be normalized.
    ///
    /// # Arguments
    /// * `incident` - The incident on the surface of an object
    /// * `normal` - The normal on the surface of an object
    ///
    /// # Returns
    /// * The radiated spectrum
    #[inline]
    pub fn radiance(&self, incident: Vector3, normal: Vector3) -> Spectrum {
        debug_assert!(is_finite(incident));
        debug_assert!(is_normalized(incident));
        debug_assert!(is_finite(normal));
        debug_assert!(is_normalized(normal));

        let dot = incident.dot(normal);

        if dot > 0.0 {
            self.emission
        } else {
            Spectrum::broadcast(0.0)
        }
    }

    pub fn radiance_light_wave(
        &self,
        incident: Vector3,
        normal: Vector3,
        light_wave_index: usize,
    ) -> Float {
        debug_assert!(is_finite(incident));
        debug_assert!(is_normalized(incident));
        debug_assert!(is_finite(normal));
        debug_assert!(is_normalized(normal));
        debug_assert!(light_wave_index < Spectrum::size());

        let dot = incident.dot(normal);
        if dot > 0.0 {
            self.emission_light_wave(light_wave_index)
        } else {
            0.0
        }
    }

    /// Samples the emitter from a given point in space.
    ///
    /// # Constraints
    /// * `point` - ALl values should be finite (neither infinite nor `NaN`).
    /// * `sample` - All values should be within `[0, 1)`.
    ///
    /// # Arguments
    /// * `point` - The point from which we sample the emitter
    /// * `sample` - A random sample
    ///
    /// # Returns
    /// * An emitter sample
    pub fn sample(&self, point: Vector3, sample: Vector2) -> EmitterSample<Spectrum> {
        debug_assert!(is_finite(point));
        debug_assert!(within_01(sample));

        let surface_sample = self.geometry.sample_surface(point, sample);

        let occlusion_tester = OcclusionTester::between(point, surface_sample.point);
        let incident = occlusion_tester.ray.direction;

        let radiance = self.radiance(-incident, surface_sample.normal);

        EmitterSample::new(radiance, incident, surface_sample.pdf, occlusion_tester)
    }

    pub fn sample_light_wave(
        &self,
        point: Vector3,
        sample: Vector2,
        light_wave_index: usize,
    ) -> EmitterSample<Float> {
        debug_assert!(is_finite(point));
        debug_assert!(within_01(sample));
        debug_assert!(light_wave_index < Spectrum::size());

        let surface_sample = self.geometry.sample_surface(point, sample);

        let occlusion_tester = OcclusionTester::between(point, surface_sample.point);
        let incident = occlusion_tester.ray.direction;

        let radiance = self.radiance_light_wave(-incident, surface_sample.normal, light_wave_index);

        EmitterSample::new(radiance, incident, surface_sample.pdf, occlusion_tester)
    }
}

impl Boundable for Emitter {
    fn bounds(&self) -> Aabb {
        self.geometry.bounds()
    }
}

impl Intersectable for Emitter {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        self.geometry.intersect(ray)
    }

    fn intersects(&self, ray: &Ray) -> bool {
        self.geometry.intersects(ray)
    }
}

/// An emitter sample consists of
/// * A `radiance` of the emitter
/// * An `incident` vector (normalized) towards the emitter
/// * A `pdf` (inside `[0, 1]`) that the emitter is hit
/// * An `occlusion tester` to test against a scene
pub struct EmitterSample<T> {
    pub radiance: T,
    pub incident: Vector3,
    pub pdf: Float,
    pub occlusion_tester: OcclusionTester,
}

impl<T> EmitterSample<T> {
    /// Creates a new emitter sample.
    ///
    /// # Constraints
    /// * `incident` - All values should be finite (neither infinite nor `NaN`).
    ///                Should be normalized.
    ///
    /// # Arguments
    /// * `radiance` - The radiance of the emitter
    /// * `incident` - The incident on the emitter
    /// * `pdf` - The pdf that the emitter is hit
    /// * `occlusion_tester` - A scene occlusion tester
    ///
    /// # Returns
    /// * Self
    pub fn new(
        radiance: T,
        incident: Vector3,
        pdf: Float,
        occlusion_tester: OcclusionTester,
    ) -> Self {
        debug_assert!(is_finite(incident));
        debug_assert!(is_normalized(incident));

        Self {
            radiance,
            incident,
            pdf,
            occlusion_tester,
        }
    }
}

/// A simple occlusion tester to test a ray against a scene.
pub struct OcclusionTester {
    ray: Ray,
}

impl OcclusionTester {
    /// Creates a new occlusion tester between the two given points.
    /// The created ray partition will be clamped to `[e, distance - e]`, with `e` denoting an epsilon
    /// and `distance` the distance between the points.
    /// This is to work around floating point imprecision that might occur in the intersection code.
    ///
    /// # Constraints
    /// * `origin` - All values should be finite (neither infinite nor `NaNN`).
    /// * `target` - All values should be finite.
    ///
    /// # Arguments
    /// * `origin` - The origin of the occlusion tester
    /// * `target` - The target to test towards
    ///
    /// # Returns
    /// * Self
    pub fn between(origin: Vector3, target: Vector3) -> Self {
        debug_assert!(is_finite(origin));
        debug_assert!(is_finite(target));
        debug_assert!(origin != target);

        let direction = target - origin;
        let distance = direction.mag();

        let mut t_start = Float::big_epsilon();
        let mut t_end = distance - Float::big_epsilon();

        if t_end < t_start {
            // edge case when distance very small
            t_start = 0.0;
            t_end = distance;
        }

        let ray = Ray::new(origin, direction / distance, t_start, t_end);

        Self { ray }
    }

    /// Tests the contained ray against the scene, whether it is unoccluded.
    ///
    /// # Arguments
    /// * `scene` - The scene to intersect against
    ///
    /// # Returns
    /// * `true` - If unoccluded
    /// * `false` - Otherwise
    pub fn unoccluded(&self, scene: &Scene) -> bool {
        !scene.is_occluded(&self.ray)
    }

    /// Tests the contained ray against the scene.
    ///
    /// # Arguments
    /// * `scene` - The scene to test against
    ///
    /// # Returns
    /// * The scene intersection (if any)
    pub fn test_get(&self, scene: &Scene) -> Option<SceneIntersection> {
        scene.intersect(&self.ray)
    }
}

/// Describes a `point`, `normal` and `pdf` of a sampled surface.
pub struct SurfaceSample {
    pub point: Vector3,
    pub normal: Vector3,
    pub pdf: Float,
}

impl SurfaceSample {
    /// Creates a new surface sample.
    ///
    /// # Constraints
    /// * `point` - All values should be finite (neither infinite nor `NaN`).
    /// * `normal` - All values should be finite.
    ///              Should be normalized.
    /// * `pdf` - Should be finite.
    ///           Should be `NaN`.
    ///
    /// # Arguments
    /// * `point` - The surface point
    /// * `normal` - The surface normal
    /// * `pdf` - The pdf of the sample
    ///
    /// # Returns
    /// * Self
    pub fn new(point: Vector3, normal: Vector3, pdf: Float) -> Self {
        debug_assert!(is_finite(point));
        debug_assert!(is_normalized(normal));
        debug_assert!(pdf >= 0.0);
        debug_assert!(!pdf.is_nan());

        Self { point, normal, pdf }
    }
}

/// Allows geometries to be sampled for a surface point.
#[typetag::serde]
pub trait Sampleable: Geometry + Send + Sync {
    /// Returns the surface area of this object.
    ///
    /// # Returns
    /// * The surface area
    fn surface_area(&self) -> Float;

    /// Samples the surface from the given point in the "solid angle" form.
    ///
    /// # Constraints
    /// * `point` - All values should be finite (neither infinite nor `NaN`).
    /// * `sample` - ALl values should be inside `[0, 1)`.
    ///
    /// # Arguments
    /// * `point` - The point at which we look at the object
    /// * `sample` - A random sample
    ///
    /// # Returns
    /// * A surface sample
    fn sample_surface(&self, origin: Vector3, sample: Vector2) -> SurfaceSample;
}
