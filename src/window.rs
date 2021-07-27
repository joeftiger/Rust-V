use crate::renderer::Renderer;
use bitflags::_core::time::Duration;
use show_image::error::{CreateWindowError, InvalidWindowId};
use show_image::event::VirtualKeyCode;
use show_image::{create_window, event, WindowOptions, WindowProxy};
use std::thread;

pub struct RenderWindow<'a> {
    window: WindowProxy,
    renderer: &'a mut Renderer,
}

impl<'a> RenderWindow<'a> {
    pub fn new<T>(name: T, renderer: &'a mut Renderer) -> Result<Self, CreateWindowError>
    where
        T: Into<String>,
    {
        let resolution = renderer.resolution();
        let width = resolution.x.min(900);
        let height = resolution.y.min(900);

        let options = WindowOptions::default()
            .set_size([width, height])
            .set_resizable(true)
            .set_preserve_aspect_ratio(true);

        let window = create_window(name, options)?;

        Ok(Self { window, renderer })
    }

    pub fn render(&mut self) -> Result<(), InvalidWindowId> {
        let render_job = self.renderer.render();

        let mut early_stop = false;

        'main: while !self.renderer.is_done() {
            for e in self.window.event_channel()? {
                if let event::WindowEvent::KeyboardInput(event) = e {
                    if event.input.state.is_pressed() {
                        if let Some(key) = event.input.key_code {
                            match key {
                                VirtualKeyCode::Escape => {
                                    early_stop = true;
                                    break 'main;
                                }
                                VirtualKeyCode::Space => {
                                    let image = self.renderer.get_image_u8();
                                    if let Err(err) = self.window.set_image("Rendering", image) {
                                        eprintln!("{}\nSkipping this image!", err);
                                    }
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_micros(500));
        }

        if early_stop {
            render_job.stop().expect("Could not stop render threads");
        } else {
            render_job.join().expect("Could not join render threads");
        }

        let image = self.renderer.get_image_u8();
        self.window
            .set_image("Final_Rendering", image)
            .expect("Could not set last image");

        // wait for user save or stop
        for event in self.window.event_channel()? {
            if let event::WindowEvent::KeyboardInput(event) = event {
                if event.input.state.is_pressed()
                    && event.input.key_code == Some(event::VirtualKeyCode::Escape)
                {
                    break;
                }
            }
        }

        Ok(())
    }
}
