/// A light wave is described by a wavelength (lambda) and an intensity (associated with amplitude).
#[derive(Copy, Clone, Default)]
pub struct LightWave {
    /// The wavelength in μm.
    pub lambda: f32,
    /// The intensity of the electric wave.
    pub intensity: f32,
}

impl LightWave {
    pub fn new(lambda: f32, intensity: f32) -> Self {
        Self { lambda, intensity }
    }

    pub fn with_intensity(&self, intensity: f32) -> Self {
        Self::new(self.lambda, intensity)
    }
}
