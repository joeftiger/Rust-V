use crate::bxdf::Type;
use crate::integrator::{direct_illumination, Integrator};
use crate::new::sensor::pixel::Pixel;
use crate::objects::SceneObject;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::{Color, Colors};
use geometry::{offset_ray_towards, Ray};

/// The Whitted integrator is a common integrator following specular reflection/transmission recursively.
#[derive(Clone)]
pub struct Whitted {
    max_depth: u32,
}

impl Whitted {
    /// Creates a new Whitted integrator.
    ///
    /// # Arguments
    /// * `depth` - The depth of recursive ray tracing
    ///
    /// # Returns
    /// * Self
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }

    fn illumination(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        depth: u32,
    ) -> Spectrum {
        let object = &intersection.object;
        let bsdf = object.bsdf();

        let mut illumination = Spectrum::black();

        if let SceneObject::Emitter(e) = object {
            illumination += e.emission; //e.radiance(&outgoing, &normal);
        }

        illumination += direct_illumination(scene, sampler, intersection, bsdf);

        let new_depth = depth + 1;
        if new_depth < self.max_depth {
            let reflection = Type::SPECULAR | Type::REFLECTION;
            let transmission = Type::SPECULAR | Type::TRANSMISSION;
            let both = reflection | transmission;
            illumination += self.integrate_typ(scene, intersection, sampler, new_depth, reflection);
            illumination +=
                self.integrate_typ(scene, intersection, sampler, new_depth, transmission);
            illumination += self.integrate_typ(scene, intersection, sampler, new_depth, both);
        }

        illumination
    }

    /// Computes a specific `Type` at the given scene intersection, calling
    /// `illumination()` with a newly generated reflected ray.
    ///
    /// # Arguments
    /// * `scene` - The scene being integratd
    /// * `intersection` - The scene intersection we illuminate
    /// * `sampler` - A sampler to generate values
    /// * `depth` - The current recursive depth (if needed)
    /// * `typ` - The typ to sample for
    ///
    /// # Returns
    /// * The illumination at the intersection
    fn integrate_typ(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        depth: u32,
        typ: Type,
    ) -> Spectrum {
        debug_assert!(depth < self.max_depth);

        let outgoing = -intersection.ray.direction;

        let bsdf = intersection.object.bsdf();
        let normal = intersection.normal;
        let sample = sampler.get_sample();

        let bxdf_sample_option = bsdf.sample(&normal, &outgoing, typ, &sample);

        let mut reflection = Spectrum::broadcast(0.0);

        if let Some(bxdf_sample) = bxdf_sample_option {
            if bxdf_sample.pdf > 0.0 && !bxdf_sample.spectrum.is_black() {
                let cos_abs = if bxdf_sample.typ.is_specular() {
                    // division of cosine omitted in specular bxdfs
                    1.0
                } else {
                    bxdf_sample.incident.dot(normal).abs()
                };

                if cos_abs != 0.0 {
                    let refl_ray = offset_ray_towards(
                        intersection.point,
                        intersection.normal,
                        bxdf_sample.incident,
                    );

                    if let Some(si) = scene.intersect(&refl_ray) {
                        let illumination = self.illumination(scene, &si, sampler, depth);
                        reflection +=
                            illumination * bxdf_sample.spectrum * (cos_abs / bxdf_sample.pdf);
                    }
                }
            }
        }

        reflection
    }
}

impl Integrator for Whitted {
    fn integrate(
        &self,
        pixel: &mut Pixel,
        scene: &Scene,
        primary_ray: &Ray,
        sampler: &dyn Sampler,
    ) {
        if let Some(i) = scene.intersect(primary_ray) {
            let illumination = self.illumination(scene, &i, sampler, 0);

            pixel.add(illumination);
        } else {
            pixel.add_black();
        }
    }
}
