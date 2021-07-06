#[macro_use]
extern crate clap;

use clap::App;

use ron::from_str;
use rust_v::renderer::Renderer;
use rust_v::serialization::Serialization;
#[cfg(feature = "show-image")]
use rust_v::RenderWindow;
use std::convert::TryInto;
use std::error::Error;

const LIVE: &str = "LIVE_WINDOW";
const VERBOSE: &str = "VERBOSE";
const INPUT: &str = "INPUT";
const FORMAT: &str = "FORMAT";
const OUTPUT: &str = "OUTPUT";
const PASSES: &str = "PASSES";
const THREADS: &str = "THREADS";

#[cfg(not(feature = "show-image"))]
fn main() -> Result<(), Box<dyn Error>> {
    create_config().run()
}

#[show_image::main]
#[cfg(feature = "show-image")]
fn main() -> Result<(), Box<dyn Error>> {
    create_config().run()?;

    show_image::exit(0);
}

fn create_config() -> CmdInput {
    #[cfg(not(feature = "show-image"))]
    let yaml = load_yaml!("cli.yml");
    #[cfg(feature = "show-image")]
    let yaml = load_yaml!("cli-live.yml");

    let matches = App::from(yaml).get_matches();

    let verbose = matches.is_present(VERBOSE);
    let live = cfg!(feature = "show-image") && matches.is_present(LIVE);
    let input = matches.value_of(INPUT).expect("No scene file given!");
    let pixel_type: PixelType = match matches.value_of(FORMAT).unwrap_or("u8").try_into() {
        Ok(format) => format,
        Err(err) => panic!("Cannot parse pixel format: {}", err),
    };
    let output = matches.value_of(OUTPUT).map(|s| s.to_string());
    let passes = matches
        .value_of(PASSES)
        .map(|string| match string.parse::<u32>() {
            Ok(p) => p,
            Err(err) => panic!("Cannot parse passes override: {}", err),
        });
    let threads = matches
        .value_of(THREADS)
        .map(|string| match string.parse::<u32>() {
            Ok(t) => t,
            Err(err) => panic!("Cannot parse threads override: {}", err),
        });

    CmdInput {
        verbose,
        live,
        input: input.to_owned(),
        pixel_type,
        output,
        passes,
        threads,
    }
}

#[derive(Debug, Clone)]
struct CmdInput {
    verbose: bool,
    live: bool,
    input: String,
    pixel_type: PixelType,
    output: Option<String>,
    passes: Option<u32>,
    threads: Option<u32>,
}

impl CmdInput {
    fn deserialize_renderer(&self) -> Renderer {
        let content =
            std::fs::read_to_string(&self.input).expect("Could not read serialization file");
        let mut serialization: Serialization =
            from_str(content.as_str()).expect("Could not parse serialization file");

        if let Some(o) = &self.output {
            if !o.is_empty() {
                serialization.config.filename = self.output.clone()
            }
        }

        if let Some(p) = self.passes {
            serialization.config.passes = p;
        }
        if self.threads.is_some() {
            serialization.config.threads = self.threads
        }

        if self.verbose {
            println!("{:#?}", serialization.config);
        }

        Renderer::new(serialization)
    }

    fn save_image(&self, renderer: &Renderer) -> Result<(), Box<dyn Error>> {
        println!("Output file: {:?}", renderer.filename());

        if let Some(path) = renderer.filename() {
            if self.verbose {
                println!("Saving image...");
            }

            match self.pixel_type {
                PixelType::U8 => renderer.get_image_u8().save(path)?,
                PixelType::U16 => renderer.get_image_u16().save(path)?,
            };

            if self.verbose {
                println!("Successfully saved image");
            }
        }

        Ok(())
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        if self.verbose {
            println!("{:#?}", self);
        }

        let mut renderer = self.deserialize_renderer();

        #[cfg(feature = "show-image")]
        if self.live {
            let mut window = RenderWindow::new("Rust-V".to_string(), &mut renderer)?;
            window.render()?;

            if self.verbose {
                println!("Closed window");
            }
        }

        if !self.live {
            let job = renderer.render();
            job.join().expect("Could not join render threads");
        }

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
