#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use anyhow::Result;

use winit::{
    dpi::LogicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use futures::executor::block_on;

mod render;
mod state;

fn main() {
    if let Err(e) = block_on(run()) {
        eprintln!("Error: {:#?}", e);
    }
}

async fn run() -> Result<()> {
    env_logger::init();
    let e_loop = EventLoop::new();

    let win = WindowBuilder::new()
        .with_inner_size(LogicalSize::<u32>::from((720, 720)))
        .with_title("cancer soot")
        .build(&e_loop)?;

    debug!("Created window");

    let mut render = render::Render::new(&win).await?;

    let size = win.inner_size();
    let mut e_state = state::State::new(size.width, size.height);

    let mut last_t = std::time::Instant::now();

    e_loop.run(move |ev, _elwt, cf| match ev {
        Event::WindowEvent {
            window_id,
            event: w_event,
        } if window_id == win.id() => match w_event {
            WindowEvent::CloseRequested => {
                *cf = ControlFlow::Exit;
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => {
                *cf = ControlFlow::Exit;
            }
            WindowEvent::CursorMoved {
                position, ..
            } => {
                e_state.section_height = 1. - position.y / e_state.size.1 as f64;
            }
            WindowEvent::Resized(new_size)
            | WindowEvent::ScaleFactorChanged {
                new_inner_size: &mut new_size,
                ..
            } => {
                render.resize(new_size);
                e_state.resize(new_size);
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            win.request_redraw();
        }
        Event::RedrawRequested(window_id) if window_id == win.id() => {
            let now = std::time::Instant::now();
            let dt = now - last_t;
            e_state.step(dt.as_secs_f64());
            last_t = now;

            match render.render(&e_state.get_rs()) {
                Ok(()) => {}
                Err(wgpu::SwapChainError::Lost) => {
                    render.resize(render.size);
                    warn!("Swap chain lost");
                }
                Err(wgpu::SwapChainError::OutOfMemory) => {
                    error!("Out of memory!");
                    *cf = ControlFlow::Exit;
                }
                Err(e) => {
                    error!("Redraw error: {:?}", e);
                }
            }
        }
        _ => {}
    });
}
