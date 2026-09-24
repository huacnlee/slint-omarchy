use std::path::Path;
use std::rc::Rc;

use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{
    Key, Platform, PlatformError, PointerEventButton, WindowAdapter, WindowEvent,
};
use slint::{ComponentHandle, LogicalPosition, PhysicalSize, Rgb8Pixel};

use crate::Gallery;

thread_local! {
    static WINDOW: Rc<MinimalSoftwareWindow> =
        MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
}

struct SnapshotPlatform;

impl Platform for SnapshotPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(WINDOW.with(Clone::clone))
    }
}

pub fn install(size: (u32, u32)) {
    slint::platform::set_platform(Box::new(SnapshotPlatform))
        .expect("set snapshot platform before creating the gallery");
    WINDOW.with(|window| window.set_size(PhysicalSize::new(size.0, size.1)));
}

pub fn save(
    app: &Gallery,
    path: &Path,
    clicks: &[(f32, f32)],
    hovers: &[(f32, f32)],
    drags: &[(f32, f32, f32, f32)],
    typed: Option<&str>,
    keys: &[String],
    wait_ms: u64,
    size: (u32, u32),
) {
    app.show().expect("show gallery in software window");
    WINDOW.with(|window| {
        window.set_size(PhysicalSize::new(size.0, size.1));
        let width = size.0 as usize;
        let height = size.1 as usize;
        let mut pixels = vec![Rgb8Pixel::default(); width * height];
        window.request_redraw();
        window.draw_if_needed(|renderer| {
            renderer.render(&mut pixels, width);
        });
        for &(x, y) in clicks {
            let position = LogicalPosition { x, y };
            window.dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            window.dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
            window.request_redraw();
            window.draw_if_needed(|renderer| {
                renderer.render(&mut pixels, width);
            });
        }
        for &(x, y) in hovers {
            window.dispatch_event(WindowEvent::PointerMoved {
                position: LogicalPosition { x, y },
            });
            window.request_redraw();
            window.draw_if_needed(|renderer| {
                renderer.render(&mut pixels, width);
            });
        }
        for &(start_x, start_y, end_x, end_y) in drags {
            window.dispatch_event(WindowEvent::PointerPressed {
                position: LogicalPosition {
                    x: start_x,
                    y: start_y,
                },
                button: PointerEventButton::Left,
            });
            for step in 1..=8 {
                let fraction = step as f32 / 8.0;
                window.dispatch_event(WindowEvent::PointerMoved {
                    position: LogicalPosition {
                        x: start_x + (end_x - start_x) * fraction,
                        y: start_y + (end_y - start_y) * fraction,
                    },
                });
            }
            window.dispatch_event(WindowEvent::PointerReleased {
                position: LogicalPosition { x: end_x, y: end_y },
                button: PointerEventButton::Left,
            });
            window.request_redraw();
            window.draw_if_needed(|renderer| {
                renderer.render(&mut pixels, width);
            });
        }
        if let Some(typed) = typed {
            for character in typed.chars() {
                let text = character.to_string().into();
                window.dispatch_event(WindowEvent::KeyPressed { text });
            }
            window.request_redraw();
            window.draw_if_needed(|renderer| {
                renderer.render(&mut pixels, width);
            });
        }
        for key in keys {
            let text = match key.as_str() {
                "Down" => Key::DownArrow.into(),
                "Up" => Key::UpArrow.into(),
                "Left" => Key::LeftArrow.into(),
                "Right" => Key::RightArrow.into(),
                "Return" => Key::Return.into(),
                "Escape" => Key::Escape.into(),
                "Tab" => Key::Tab.into(),
                "Home" => Key::Home.into(),
                "End" => Key::End.into(),
                "Backspace" => Key::Backspace.into(),
                other => other.into(),
            };
            window.dispatch_event(WindowEvent::KeyPressed { text });
            window.request_redraw();
            window.draw_if_needed(|renderer| {
                renderer.render(&mut pixels, width);
            });
        }
        if wait_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(wait_ms.min(10_000)));
            slint::platform::update_timers_and_animations();
            window.request_redraw();
            window.draw_if_needed(|renderer| {
                renderer.render(&mut pixels, width);
            });
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create screenshot directory");
        }
        let file = std::fs::File::create(path).expect("create PNG screenshot");
        let mut encoder = png::Encoder::new(file, width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("write PNG header");
        let bytes = pixels
            .iter()
            .flat_map(|pixel| [pixel.r, pixel.g, pixel.b])
            .collect::<Vec<_>>();
        writer.write_image_data(&bytes).expect("write PNG pixels");
    });
}
