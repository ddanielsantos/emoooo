use std::sync::Arc;
use std::time::{Duration, Instant};
use pixels::{Pixels, SurfaceTexture};
use pixels::wgpu::PresentMode;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

mod chip8;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const FRAME_TIME: Duration = Duration::from_millis(1000 / 240);

struct FPSDebug {
    last_frame: Instant,
    last_fps_check: Instant,
    frame_counter: u32,
}

impl Default for FPSDebug {
    fn default() -> FPSDebug {
        FPSDebug {
            last_frame: Instant::now(),
            last_fps_check: Instant::now(),
            frame_counter: 0,
        }
    }
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    fps_debug: FPSDebug
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attr = Window::default_attributes()
            .with_title("chip8-emoooo")
            .with_inner_size(LogicalSize::new(WIDTH * 10, HEIGHT * 10));

        let window = Arc::new(event_loop.create_window(window_attr).unwrap());
        self.pixels = {
            let win_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(win_size.width, win_size.height, window.clone());

            match Pixels::new(WIDTH, HEIGHT, surface_texture) {
                Ok(mut p) => {
                    window.request_redraw();
                    p.set_present_mode(PresentMode::Immediate);
                    Some(p)
                },
                Err(_e) => {
                    event_loop.exit();
                    None
                }
            }
        };

        self.window = Some(window.clone());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {

        match event {
            WindowEvent::CloseRequested => {
                println!("Close Requested");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let frame_time = now.duration_since(self.fps_debug.last_frame);
                self.fps_debug.last_frame = now;

                if let Some(pixels) = self.pixels.as_mut() {
                    let frame = pixels.frame_mut();
                    for chunk in frame.chunks_exact_mut(4) {
                        chunk.copy_from_slice(&[0x00, 0xF0, 0x00, 0xFF]); // black
                    }

                    if let Err(e) = pixels.render() {
                        eprintln!("pixels.render() failed: {e}");
                        event_loop.exit();
                    }
                }

                self.fps_debug.frame_counter += 1;
                println!("Frame time: {:.2?}", frame_time);
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            let now = Instant::now();
            if now.duration_since(self.fps_debug.last_frame) >= FRAME_TIME {
                window.request_redraw();
            }

            // 2️⃣ Once per second, show FPS
            if now.duration_since(self.fps_debug.last_fps_check) >= Duration::from_secs(1) {
                println!("FPS: {}", self.fps_debug.frame_counter);
                self.fps_debug.frame_counter = 0;
                self.fps_debug.last_fps_check = now;
            }
        }
    }
}

fn main() -> Result<(), impl std::error::Error> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)
}
