use crate::renderer::Renderer;
use std::convert::TryInto;
use ultraviolet::UVec2;

/// # Summary
/// A configuration stores all parameters for the renderer.
#[derive(Debug, Clone)]
pub struct Configuration {
    pub verbose: bool,
    pub resolution: UVec2,
    pub depth: u32,
    pub passes: u32,
    pub block_size: u32,
    pub live: bool,
    pub threads: u32,
    pub output: Option<String>,
    pub pixel_type: PixelType,
    pub integrator_type: IntegratorType,
    pub demo_type: DemoType,
}

impl Configuration {
    pub fn create_renderer(&self) -> Renderer {
        // let (scene, camera) = match self.demo_type {
        //     DemoType::SphereScene => SphereScene::create(self.width, self.height),
        //     DemoType::CornellScene => CornellScene::create(self.width, self.height),
        //     DemoType::DebugScene => DebugScene::create(self.width, self.height),
        // };

        unimplemented!()
    }
}

/// # Summary
/// Represents the pixel type to save.
#[derive(Debug, Clone)]
pub enum PixelType {
    U8,
    U16,
}

impl TryInto<PixelType> for &str {
    type Error = String;

    fn try_into(self) -> Result<PixelType, Self::Error> {
        match self {
            "u8" | "U8" => Ok(PixelType::U8),
            "u16" | "U16" => Ok(PixelType::U16),
            _ => Err(self.to_string()),
        }
    }
}

/// # Summary
/// Represents the scene integrator.
#[derive(Debug, Clone)]
pub enum IntegratorType {
    Debug,
    Whitted,
    Path,
}

impl TryInto<IntegratorType> for &str {
    type Error = String;

    fn try_into(self) -> Result<IntegratorType, Self::Error> {
        match self {
            "debug" | "Debug" | "DEBUG" => Ok(IntegratorType::Debug),
            "whitted" | "Whitted" | "WHITTED" => Ok(IntegratorType::Whitted),
            "path" | "Path" | "PATH" => Ok(IntegratorType::Path),
            _ => Err(self.to_string()),
        }
    }
}

/// # Summary
/// Represents a demo scene to load.
#[derive(Debug, Clone)]
pub enum DemoType {
    SphereScene,
    CornellScene,
    DebugScene,
}

impl TryInto<DemoType> for &str {
    type Error = String;

    fn try_into(self) -> Result<DemoType, Self::Error> {
        match self {
            "spheres" | "Spheres" | "SPHERES" => Ok(DemoType::SphereScene),
            "cornell" | "Cornell" | "CORNELL" => Ok(DemoType::CornellScene),
            "debug" | "Debug" | "DEBUG" => Ok(DemoType::DebugScene),
            _ => Err(self.to_string()),
        }
    }
}
