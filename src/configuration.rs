use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub verbose: bool,
    pub width: u32,
    pub height: u32,
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
    // pub fn create_renderer(&self) -> Renderer {
    //
    // }
}

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
