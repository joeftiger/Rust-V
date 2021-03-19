use crate::bxdf::{
    BxDF, FresnelSpecular, LambertianReflection, LambertianTransmission, OrenNayar,
    SpecularReflection, SpecularTransmission, Type,
};
use crate::Spectrum;
use serde::{Deserialize, Serialize};
use ultraviolet::Vec3;

#[derive(Serialize, Deserialize)]
pub enum BSDFType {
    LReflection(LambertianReflection),
    LTransmission(LambertianTransmission),
    OrenNayar(OrenNayar),
    SReflection(SpecularReflection),
    STransmission(SpecularTransmission),
    SFresnel(FresnelSpecular),
}

impl BxDF for BSDFType {
    fn get_type(&self) -> Type {
        match self {
            BSDFType::LReflection(t) => t.get_type(),
            BSDFType::LTransmission(t) => t.get_type(),
            BSDFType::OrenNayar(t) => t.get_type(),
            BSDFType::SReflection(t) => t.get_type(),
            BSDFType::STransmission(t) => t.get_type(),
            BSDFType::SFresnel(t) => t.get_type(),
        }
    }

    fn evaluate(&self, incident: &Vec3, outgoing: &Vec3) -> Spectrum {
        match self {
            BSDFType::LReflection(t) => t.evaluate(incident, outgoing),
            BSDFType::LTransmission(t) => t.evaluate(incident, outgoing),
            BSDFType::OrenNayar(t) => t.evaluate(incident, outgoing),
            BSDFType::SReflection(t) => t.evaluate(incident, outgoing),
            BSDFType::STransmission(t) => t.evaluate(incident, outgoing),
            BSDFType::SFresnel(t) => t.evaluate(incident, outgoing),
        }
    }

    fn evaluate_light_wave(
        &self,
        incident: &Vec3,
        outgoing: &Vec3,
        light_wave_index: usize,
    ) -> f32 {
        match self {
            BSDFType::LReflection(t) => t.evaluate_light_wave(incident, outgoing, light_wave_index),
            BSDFType::LTransmission(t) => {
                t.evaluate_light_wave(incident, outgoing, light_wave_index)
            }
            BSDFType::OrenNayar(t) => t.evaluate_light_wave(incident, outgoing, light_wave_index),
            BSDFType::SReflection(t) => t.evaluate_light_wave(incident, outgoing, light_wave_index),
            BSDFType::STransmission(t) => {
                t.evaluate_light_wave(incident, outgoing, light_wave_index)
            }
            BSDFType::SFresnel(t) => t.evaluate_light_wave(incident, outgoing, light_wave_index),
        }
    }
}
