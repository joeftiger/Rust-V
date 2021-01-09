use crate::renderer::Renderer;
use bitflags::_core::time::Duration;
use show_image::{make_window_full, KeyCode, Window, WindowOptions};

pub struct RenderWindow {
    window: Window,
    renderer: Renderer,
}

impl RenderWindow {
    pub fn new(name: String, renderer: Renderer) -> Result<Self, String> {
        let width = 900;
        let height = 900;

        let options = WindowOptions::default()
            .set_name(name)
            .set_width(width)
            .set_height(height)
            .set_resizable(true)
            .set_preserve_aspect_ratio(true);

        let window = make_window_full(options)?;

        Ok(Self { window, renderer })
    }

    pub fn render(&mut self) {
        let wait_key = Duration::from_millis(500);
        let render_job = self.renderer.render();

        while let Ok(event) = self.window.wait_key(wait_key) {
            if let Some(e) = event {
                if e.key == KeyCode::Escape {
                    render_job.stop().expect("Could not stop render threads");
                    return;
                }
            }

            if self.renderer.is_done() {
                break;
            }

            let image = self.renderer.get_image_u8();
            if let Some(err) = self.window.set_image(image, "Rendering").err() {
                eprintln!("{}\nSkipping this image!", err);
            }
        }

        render_job.join().expect("Could not join render threads");

        let image = self.renderer.get_image_u8();
        self.window
            .set_image(image, "Rendering")
            .expect("Could not set last image");

        // wait for user save or stop
        while let Ok(event) = self.window.wait_key(wait_key) {
            if let Some(e) = event {
                if e.key == KeyCode::Escape {
                    break;
                }
            }
        }

        // Make sure all background tasks are stopped cleanly.
        show_image::stop().unwrap();
    }
}
