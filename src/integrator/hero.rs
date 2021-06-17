use crate::bxdf::Type;
use crate::integrator::{direct_illumination_buf, DirectLightStrategy, Integrator};
use crate::objects::SceneObject;
use crate::samplers::spectral_samplers::SpectralSampler;
use crate::samplers::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::sensor::pixel::Pixel;
use crate::Float;
use geometry::{offset_ray_towards, Ray};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Hero {
    max_depth: u32,
    light_wave_samples: u32,
    strategy: DirectLightStrategy,
}

impl Hero {
    fn trace(
        &self,
        scene: &Scene,
        mut hit: SceneIntersection,
        sampler: Sampler,
        indices: &[usize],
        illumination: &mut [Float],
        throughput: &mut [Float],
    ) {
        assert_eq!(indices.len(), illumination.len());
        assert_eq!(indices.len(), throughput.len());

        let mut specular = false;
        for bounce in 0..self.max_depth {
            let outgoing = -hit.ray.direction;
            let normal = hit.normal;
            let bsdf = hit.object.bsdf();

            // immediately hitting emitter?
            if bounce == 0 {
                if let SceneObject::Emitter(e) = &hit.object {
                    for i in 0..indices.len() {
                        illumination[i] = e.emission[i];
                    }
                    break;
                }
            } else if specular {
                if let SceneObject::Emitter(e) = &hit.object {
                    for i in 0..indices.len() {
                        illumination[i] = throughput[i] * e.emission[i];
                    }
                    break;
                }
            }

            // add direction illumination
            direct_illumination_buf(
                scene,
                sampler,
                self.strategy,
                &hit,
                bsdf,
                indices,
                illumination,
            );

            if let Some(bxdf_sample) =
                bsdf.sample_buf(normal, outgoing, Type::ALL, sampler.get_sample(), indices)
            {
                if bxdf_sample.pdf == 0.0 || bxdf_sample.spectrum.iter().any(|&s| s == 0.0) {
                    break;
                }

                specular = bxdf_sample.typ.is_specular();
                let cos_abs = if specular {
                    // division of cosine omitted in specular bxdfs
                    1.0
                } else {
                    bxdf_sample.incident.dot(normal).abs()
                };

                for i in 0..indices.len() {
                    let before = throughput[i];
                    throughput[i] *= bxdf_sample.spectrum[i] * cos_abs
                        / bxdf_sample.pdf
                        / indices.len() as Float;
                    let after = throughput[i];

                    if before <= after {
                        println!("Before: {:.3}\tAfter: {:.3}", before, after);
                    }
                }

                let ray = offset_ray_towards(hit.point, hit.normal, bxdf_sample.incident);
                match scene.intersect(&ray) {
                    Some(i) => hit = i,
                    None => break,
                }
            } else {
                break;
            }
        }
    }
}

#[typetag::serde]
impl Integrator for Hero {
    fn integrate(&self, pixel: &mut Pixel, scene: &Scene, primary_ray: &Ray, sampler: Sampler) {
        if let Some(hit) = scene.intersect(primary_ray) {
            let mut indices = vec![0; self.light_wave_samples as usize];
            let mut illumination = vec![0.0; self.light_wave_samples as usize];
            let mut throughput = vec![1.0; self.light_wave_samples as usize];

            SpectralSampler::Hero.fill_samples(&mut indices);

            self.trace(
                scene,
                hit,
                sampler,
                &indices,
                &mut illumination,
                &mut throughput,
            );

            for (&index, &lambda) in indices.iter().zip(illumination.iter()) {
                pixel.add_light_wave(lambda, index);
            }
        } else {
            pixel.add_black();
        }
    }
}
