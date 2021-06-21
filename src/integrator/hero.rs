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
    direct_light_strategy: DirectLightStrategy,
    spectral_sampler: SpectralSampler,
}

impl Hero {
    #[allow(clippy::needless_range_loop)] // clippy is stupid here
    fn trace(
        &self,
        scene: &Scene,
        mut hit: SceneIntersection,
        sampler: Sampler,
        indices: &[usize],
        illumination: &mut [Float],
        throughput: &mut [Float],
    ) {
        let buf_size = indices.len();
        assert_eq!(buf_size, illumination.len());
        assert_eq!(buf_size, throughput.len());

        let mut specular = false;
        for bounce in 0..self.max_depth {
            let outgoing = -hit.ray.direction;
            let normal = hit.normal;
            let bsdf = hit.object.bsdf();

            // immediately hitting emitter?
            if bounce == 0 {
                if let SceneObject::Emitter(e) = &hit.object {
                    for i in 0..buf_size {
                        illumination[i] = e.emission[i];
                    }
                    break;
                }
            } else if specular {
                if let SceneObject::Emitter(e) = &hit.object {
                    for i in 0..buf_size {
                        illumination[i] = throughput[i] * e.emission[i];
                    }
                    break;
                }
            }

            // add direction illumination
            direct_illumination_buf(
                scene,
                sampler,
                self.direct_light_strategy,
                &hit,
                bsdf,
                indices,
                illumination,
                throughput,
            );

            if let Some(bxdf_sample) =
                bsdf.sample_buf(normal, outgoing, Type::ALL, sampler.get_sample(), indices)
            {
                if bxdf_sample.pdf == 0.0 || bxdf_sample.spectrum.iter().all(|&s| s == 0.0) {
                    break;
                }

                specular = bxdf_sample.typ.is_specular();
                let cos_abs = if specular {
                    // division of cosine omitted in specular bxdfs
                    1.0
                } else {
                    bxdf_sample.incident.dot(normal).abs()
                };

                for i in 0..buf_size {
                    throughput[i] *= bxdf_sample.spectrum[i] * cos_abs / bxdf_sample.pdf;
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

            self.spectral_sampler.fill_samples(&mut indices);

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
