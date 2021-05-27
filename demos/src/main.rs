#[allow(unused_imports)]
use crate::demo_scenes::{CornellScene, Demo, PrismScene};
use color::{AsColor, Color, Colors, Spectrum, Srgb, Xyz};
use ron::ser::{to_string_pretty, PrettyConfig};
use show_image::event::VirtualKeyCode;
use show_image::{create_window, event, ImageInfo, ImageView, WindowOptions};
use ultraviolet::UVec2;

#[cfg(not(feature = "f64"))]
type Float = f32;
#[cfg(not(feature = "f64"))]
type Rotation3 = ultraviolet::Rotor3;
#[cfg(feature = "f64")]
type Float = f64;
#[cfg(feature = "f64")]
type Rotation3 = ultraviolet::DRotor3;

mod demo_scenes;

// fn main() {
//     let scene = CornellScene::create();
//
//     let serialization =
//         to_string_pretty(&scene, PrettyConfig::default()).expect("Could not serialize to RON");
//
//     std::fs::write("./scenes/dragon_4.ron", serialization).expect("Could not write to RON file");
// }

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = WindowOptions::default().set_size([500, 500]);

    let window = create_window("Test", options)?;

    let colors = Colors::variants();
    let mut i = 0;

    loop {
        let c = colors[i];
        let spectrum = Spectrum::from(c);
        let xyz = Xyz::from(c);
        let srgb = Srgb::from(c);
        let xyz_conv = Xyz::from(spectrum);
        let srgb_conv = Srgb::from(spectrum);
        let srgb_xyz = Srgb::from(xyz);

        println!("{:?}", c);
        println!("-- {:?}", spectrum);
        println!("-- {:?}", xyz);
        println!("-- {:?} (converted)", xyz_conv);
        println!("-- {:?}", srgb);
        println!("-- {:?} (converted from xyz)\n", srgb_xyz);
        println!("-- {:?} (converted)\n", srgb_conv);
        println!("{:?}", srgb_conv / srgb);

        // convert float to u8
        let scale = Srgb::broadcast(2u32.pow(8) as Float);
        let srgb_bytes = srgb * scale;
        let srgb_conv_bytes = srgb_conv * scale;
        let data = [
            srgb_bytes[0] as u8,
            srgb_bytes[1] as u8,
            srgb_bytes[2] as u8,
            srgb_conv_bytes[0] as u8,
            srgb_conv_bytes[1] as u8,
            srgb_conv_bytes[2] as u8,
        ];

        let image = ImageView::new(ImageInfo::rgb8(2, 1), &data);
        window.set_image(format!("{:?}", c), image)?;

        for event in window.event_channel()? {
            if let event::WindowEvent::KeyboardInput(event) = event {
                if event.input.state.is_pressed() {
                    match event.input.key_code {
                        Some(event::VirtualKeyCode::Escape) => return Ok(()),
                        Some(event::VirtualKeyCode::Left) => {
                            i = if i == 0 { i } else { i - 1 };
                            break;
                        }
                        Some(event::VirtualKeyCode::Right) => {
                            i = if i == (colors.len() - 1) { i } else { i + 1 };
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
