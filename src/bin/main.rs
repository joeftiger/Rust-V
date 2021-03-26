#[macro_use]
extern crate clap;

use clap::App;

use ron::from_str;
use rust_v::integrator::{DebugNormals, Integrator, Path, PathEnhanced, SpectralPath, Whitted};
use rust_v::renderer::Renderer;
use rust_v::sampler::{NoOpSampler, RandomSampler, Sampler};
use rust_v::scene::Scene;
use rust_v::RenderConfig;
#[cfg(feature = "live-window")]
use rust_v::RenderWindow;
use std::convert::TryInto;
use std::sync::Arc;

const LIVE: &str = "LIVE_WINDOW";
const VERBOSE: &str = "VERBOSE";
#[allow(dead_code)]
const INPUT: &str = "INPUT";
const OUTPUT: &str = "OUTPUT";
const PASSES: &str = "PASSES";
const BLOCK_SIZE: &str = "BLOCK_SIZE";
const DEPTH: &str = "DEPTH";
const WIDTH: &str = "WIDTH";
const HEIGHT: &str = "HEIGHT";
const FORMAT: &str = "FORMAT";
const INTEGRATOR_BACKEND: &str = "INTEGRATOR_BACKEND";
const THREADS: &str = "THREADS";

fn main() -> Result<(), String> {
    create_config().run()
}

fn create_config() -> MainConfig {
    #[cfg(not(feature = "live-window"))]
    let yaml = load_yaml!("cli.yml");
    #[cfg(feature = "live-window")]
    let yaml = load_yaml!("cli-live.yml");

    let matches = App::from_yaml(yaml).get_matches();

    let input = matches.value_of(INPUT).expect("No scene file given!");

    let verbose = matches.is_present(VERBOSE);

    let width = match matches.value_of(WIDTH).unwrap_or("900").parse() {
        Ok(width) => width,
        Err(err) => panic!("Cannot parse width: {}", err),
    };
    let height = match matches.value_of(HEIGHT).unwrap_or("900").parse() {
        Ok(height) => height,
        Err(err) => panic!("Cannot parse height: {}", err),
    };
    let depth = match matches.value_of(DEPTH).unwrap_or("6").parse() {
        Ok(depth) => depth,
        Err(err) => panic!("Cannot parse depth: {}", err),
    };
    let passes = match matches.value_of(PASSES).unwrap_or("1").parse() {
        Ok(passes) => passes,
        Err(err) => panic!("Cannot parse passes: {}", err),
    };
    let block_size = match matches.value_of(BLOCK_SIZE).unwrap_or("8").parse() {
        Ok(block_size) => block_size,
        Err(err) => panic!("Cannot parse block size: {}", err),
    };
    let live = cfg!(feature = "live-window") && matches.is_present(LIVE);
    let threads = match matches
        .value_of(THREADS)
        .unwrap_or(&num_cpus::get().to_string())
        .parse()
    {
        Ok(threads) => threads,
        Err(err) => panic!("Cannot parse threads: {}", err),
    };
    let pixel_type: PixelType = match matches.value_of(FORMAT).unwrap_or("u8").try_into() {
        Ok(format) => format,
        Err(err) => panic!("Cannot parse pixel format: {}", err),
    };
    let integrator_type: IntegratorType = match matches
        .value_of(INTEGRATOR_BACKEND)
        .unwrap_or("whitted")
        .try_into()
    {
        Ok(integrator) => integrator,
        Err(err) => panic!("Cannot parse integrator backend: {}", err),
    };

    let output = if let Some(o) = matches.value_of(OUTPUT) {
        o.to_string()
    } else {
        format!(
            "{}.png",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        )
    };
    let output = if output.is_empty() {
        None
    } else {
        Some(output)
    };

    let render_config = RenderConfig::new(width, height, depth, passes, block_size, threads);

    MainConfig {
        render_config,
        verbose,
        live,
        input: input.to_owned(),
        output,
        pixel_type,
        integrator_type,
    }
}

#[derive(Debug, Clone)]
struct MainConfig {
    pub render_config: RenderConfig,
    pub verbose: bool,
    pub live: bool,
    pub input: String,
    pub output: Option<String>,
    pub pixel_type: PixelType,
    pub integrator_type: IntegratorType,
}

impl MainConfig {
    fn create_renderer(&self) -> Renderer {
        let content = std::fs::read_to_string(&self.input).expect("Could not read scene file");
        let mut scene: Scene = from_str(content.as_str()).expect("Could not parse scene file");
        scene.init();

        let integrator: Arc<dyn Integrator> = match self.integrator_type {
            IntegratorType::Debug => Arc::new(DebugNormals),
            IntegratorType::Whitted => Arc::new(Whitted::new(self.render_config.depth)),
            IntegratorType::Path => Arc::new(Path::new(self.render_config.depth)),
            IntegratorType::PathEnhanced => Arc::new(PathEnhanced::new(
                self.render_config.depth,
                self.render_config.depth * 4,
            )),
            IntegratorType::SpectralPath => Arc::new(SpectralPath::new(
                self.render_config.depth,
                self.render_config.depth * 4,
            )),
        };

        let sampler: Arc<dyn Sampler> = match self.integrator_type {
            IntegratorType::Debug => Arc::new(NoOpSampler),
            _ => Arc::new(RandomSampler::default()),
        };

        Renderer::new(scene, sampler, integrator, self.render_config)
    }

    fn save_image(&self, renderer: &Renderer) -> Result<(), String> {
        if let Some(path) = &self.output {
            if self.verbose {
                println!("Saving image...");
            }

            match self.pixel_type {
                PixelType::U8 => renderer
                    .get_image_u8()
                    .save(path)
                    .map_err(|e| format!("Unable to save image: {}", e))?,
                PixelType::U16 => renderer
                    .get_image_u16()
                    .save(path)
                    .map_err(|e| format!("Unable to save image: {}", e))?,
            };

            if self.verbose {
                println!("Successfully saved image");
            }
        }

        Ok(())
    }

    pub fn run(&self) -> Result<(), String> {
        if self.verbose {
            println!("{:#?}", self);
        }

        let mut renderer = self.create_renderer();

        #[cfg(feature = "live-window")]
        if self.live {
            let mut window = RenderWindow::new("Rust-V".to_string(), renderer)?;
            window.render();

            if self.verbose {
                println!("Closed window");
            }

            return Ok(());
        }

        let job = renderer.render();
        job.join().expect("Could not join render threads");

        self.save_image(&renderer)
    }
}

/// Represents the pixel type to save.
#[derive(Debug, Clone)]
pub enum PixelType {
    U8,
    U16,
}

impl TryInto<PixelType> for &str {
    type Error = String;

    fn try_into(self) -> Result<PixelType, Self::Error> {
        match self.to_lowercase().as_str() {
            "u8" => Ok(PixelType::U8),
            "u16" => Ok(PixelType::U16),
            _ => Err(self.to_string()),
        }
    }
}

/// Represents the scene integrator.
#[derive(Debug, Clone)]
pub enum IntegratorType {
    Debug,
    Whitted,
    Path,
    PathEnhanced,
    SpectralPath,
}

impl TryInto<IntegratorType> for &str {
    type Error = String;

    fn try_into(self) -> Result<IntegratorType, Self::Error> {
        match self.to_lowercase().as_str() {
            "debug" => Ok(IntegratorType::Debug),
            "whitted" => Ok(IntegratorType::Whitted),
            "path" => Ok(IntegratorType::Path),
            "pathenhanced" => Ok(IntegratorType::PathEnhanced),
            "spectralpath" => Ok(IntegratorType::SpectralPath),
            _ => Err(self.to_string()),
        }
    }
}
