#[macro_use]
extern crate clap;

use clap::App;

use rust_v::demo_scenes::{CornellScene, DebugScene, DebugSphereScene, DemoScene, SphereScene};
use rust_v::integrator::{DebugNormals, Integrator, Path, Whitted};
use rust_v::renderer::Renderer;
use rust_v::sampler::{NoOpSampler, RandomSampler, Sampler};
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

    let app_matches = App::from_yaml(yaml).get_matches();
    // let demo = if let Some(spheres) = app_matches.subcommand_matches(SPHERE_SCENE) {
    //     (spheres, DemoType::SphereScene)
    // } else if let Some(cornell) = app_matches.subcommand_matches(CORNELL_SCENE) {
    //     (cornell, DemoType::CornellScene)
    // } else if let Some(debug) = app_matches.subcommand_matches(DEBUG_SCENE) {
    //     (debug, DemoType::DebugScene)
    // } else {
    //     panic!("Currently we only support the subcommands (spheres, cornell, debug)!");
    // };

    let (demo_type, matches) = match app_matches.subcommand() {
        (name, Some(m)) => {
            let typ: DemoType = match name.try_into() {
                Ok(t) => t,
                Err(err) => panic!("Cannot parse demo type: {}", err),
            };

            (typ, m)
        }
        (_, None) => panic!("No subcommand given"),
    };
    // let (demo_type_name, Some(matches)) = app_matches.subcommand();
    // let demo_type: DemoType = match demo_type_name.try_into() {
    //     Ok(typ) => typ,
    //     Err(err) => panic!("Cannot parse demo type: {}", err),
    // };

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
        output,
        pixel_type,
        integrator_type,
        demo_type,
    }
}

#[derive(Debug, Clone)]
struct MainConfig {
    pub render_config: RenderConfig,
    pub verbose: bool,
    pub live: bool,
    pub output: Option<String>,
    pub pixel_type: PixelType,
    pub integrator_type: IntegratorType,
    pub demo_type: DemoType,
}

impl MainConfig {
    fn create_renderer(&self) -> Renderer {
        let (scene, camera) = match self.demo_type {
            DemoType::SphereScene => SphereScene::create(self.render_config.resolution),
            DemoType::CornellScene => CornellScene::create(self.render_config.resolution),
            DemoType::DebugScene => DebugScene::create(self.render_config.resolution),
            DemoType::DebugSphereScene => DebugSphereScene::create(self.render_config.resolution),
        };

        let integrator: Arc<dyn Integrator> = match self.integrator_type {
            IntegratorType::Debug => Arc::new(DebugNormals),
            IntegratorType::Whitted => Arc::new(Whitted::new(self.render_config.depth)),
            IntegratorType::Path => Arc::new(Path::new(self.render_config.depth)),
        };

        let sampler: Arc<dyn Sampler> = match self.integrator_type {
            IntegratorType::Debug => Arc::new(NoOpSampler),
            _ => Arc::new(RandomSampler::default()),
        };

        Renderer::new(scene, camera, sampler, integrator, self.render_config)
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
        {
            if self.live {
                let mut window = RenderWindow::new("Rust-V".to_string(), renderer)?;
                window.render();

                if self.verbose {
                    println!("Closed window");
                }

                return Ok(());
            }
        }

        let job = renderer.render();
        job.join().expect("Could not join render threads");

        self.save_image(&renderer)
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
    DebugSphereScene,
}

impl TryInto<DemoType> for &str {
    type Error = String;

    fn try_into(self) -> Result<DemoType, Self::Error> {
        match self {
            "spheres" | "Spheres" | "SPHERES" => Ok(DemoType::SphereScene),
            "cornell" | "Cornell" | "CORNELL" => Ok(DemoType::CornellScene),
            "debug" | "Debug" | "DEBUG" => Ok(DemoType::DebugScene),
            "debugsphere" | "DebugSphere" | "DEBUGSPHERE" => Ok(DemoType::DebugSphereScene),
            _ => Err(self.to_string()),
        }
    }
}
