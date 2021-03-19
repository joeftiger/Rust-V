use crate::bxdf::BSDF;

use crate::debug_utils::{is_finite, is_normalized, within_01};
use crate::objects::receiver::ReceiverExt;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::{Color, IndexSpectral};
use geometry::{Aabb, Boundable, Geometry, Intersectable, Intersection, Ray};
use ultraviolet::{Vec2, Vec3};
use utility::floats::BIG_EPSILON;

/// An emitter is a receiver that also emits light.
pub trait EmitterExt: ReceiverExt {
    /// Returns the emission of this emitter.
    ///
    /// # Returns
    /// * The emission
    fn emission(&self) -> Spectrum;

    fn emission_light_wave(&self, light_wave_index: usize) -> f32;

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
    fn radiance(&self, incident: &Vec3, normal: &Vec3) -> Spectrum {
        debug_assert!(is_finite(incident));
        debug_assert!(is_normalized(incident));
        debug_assert!(is_finite(normal));
        debug_assert!(is_normalized(normal));

        let dot = incident.dot(*normal);

        if dot > 0.0 {
            self.emission()
        } else {
            Spectrum::new_const(0.0)
        }
    }

    fn radiance_light_wave(&self, incident: &Vec3, normal: &Vec3, light_wave_index: usize) -> f32 {
        debug_assert!(is_finite(incident));
        debug_assert!(is_normalized(incident));
        debug_assert!(is_finite(normal));
        debug_assert!(is_normalized(normal));
        debug_assert!(light_wave_index < Spectrum::size());

        let dot = incident.dot(*normal);
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
    fn sample(&self, point: &Vec3, sample: &Vec2) -> EmitterSample<Spectrum>;

    fn sample_light_wave(
        &self,
        point: &Vec3,
        sample: &Vec2,
        light_wave_index: usize,
    ) -> EmitterSample<f32>;
}

/// An emitter is similar to a receiver, consisting of a geometry and a BSDF. Additionally, the
/// emitter also has an emission.
pub struct Emitter<T> {
    geometry: T,
    bsdf: BSDF,
    emission: Spectrum,
}

impl<T> Emitter<T> {
    /// Creates a new emitter.
    ///
    /// # Arguments
    /// * `geometry` - The geometry of the emitter
    /// * `bsdf` - The BSDF of the emitter
    /// * `emission` - The emission of the emitter
    ///
    /// # Returns
    /// * Self
    pub fn new(geometry: T, bsdf: BSDF, emission: Spectrum) -> Self {
        Self {
            geometry,
            bsdf,
            emission,
        }
    }
}

impl<T> EmitterExt for Emitter<T>
where
    T: Sampleable,
{
    fn emission(&self) -> Spectrum {
        self.emission
    }

    fn emission_light_wave(&self, light_wave_index: usize) -> f32 {
        debug_assert!(light_wave_index < Spectrum::size());

        self.emission.index_spectral(light_wave_index)
    }

    fn sample(&self, point: &Vec3, sample: &Vec2) -> EmitterSample<Spectrum> {
        debug_assert!(is_finite(point));
        debug_assert!(within_01(sample));

        let surface_sample = self.geometry.sample_surface(point, sample);

        let occlusion_tester = OcclusionTester::between(*point, surface_sample.point);
        let incident = occlusion_tester.ray.direction;

        let radiance = self.radiance(&-incident, &surface_sample.normal);

        EmitterSample::new(radiance, incident, surface_sample.pdf, occlusion_tester)
    }

    fn sample_light_wave(
        &self,
        point: &Vec3,
        sample: &Vec2,
        light_wave_index: usize,
    ) -> EmitterSample<f32> {
        debug_assert!(is_finite(point));
        debug_assert!(within_01(sample));
        debug_assert!(light_wave_index < Spectrum::size());

        let surface_sample = self.geometry.sample_surface(point, sample);

        let occlusion_tester = OcclusionTester::between(*point, surface_sample.point);
        let incident = occlusion_tester.ray.direction;

        let radiance =
            self.radiance_light_wave(&-incident, &surface_sample.normal, light_wave_index);

        EmitterSample::new(radiance, incident, surface_sample.pdf, occlusion_tester)
    }
}

impl<T> ReceiverExt for Emitter<T>
where
    T: Geometry + Send + Sync,
{
    fn bsdf(&self) -> &BSDF {
        &self.bsdf
    }
}

impl<T> Boundable for Emitter<T>
where
    T: Boundable,
{
    fn bounds(&self) -> Aabb {
        self.geometry.bounds()
    }
}

impl<T> Intersectable for Emitter<T>
where
    T: Intersectable,
{
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
    pub incident: Vec3,
    pub pdf: f32,
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
    pub fn new(radiance: T, incident: Vec3, pdf: f32, occlusion_tester: OcclusionTester) -> Self {
        debug_assert!(is_finite(&incident));
        debug_assert!(is_normalized(&incident));

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
    pub(crate) ray: Ray,
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
    pub fn between(origin: Vec3, target: Vec3) -> Self {
        debug_assert!(is_finite(&origin));
        debug_assert!(is_finite(&target));
        debug_assert!(origin != target);

        let direction = target - origin;
        let distance = direction.mag();

        let mut t_start = BIG_EPSILON;
        let mut t_end = distance - BIG_EPSILON;

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
    pub point: Vec3,
    pub normal: Vec3,
    pub pdf: f32,
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
    pub fn new(point: Vec3, normal: Vec3, pdf: f32) -> Self {
        debug_assert!(is_finite(&point));
        debug_assert!(is_normalized(&normal));
        debug_assert!(pdf >= 0.0);
        debug_assert!(!pdf.is_nan());

        Self { point, normal, pdf }
    }
}

/// Allows geometries to be sampled for a surface point.
pub trait Sampleable: Geometry + Send + Sync {
    /// Returns the surface area of this object.
    ///
    /// # Returns
    /// * The surface area
    fn surface_area(&self) -> f32;

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
    fn sample_surface(&self, origin: &Vec3, sample: &Vec2) -> SurfaceSample;
}
