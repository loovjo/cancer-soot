#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use anyhow::Result;

use winit::{
    dpi::LogicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use futures::executor::block_on;

mod render;
mod state;
mod easing;
mod laz;

use easing::Easing;

fn main() -> Result<()> {
    let (mut env, sum_id) = laz::example_env();

    println!("{:?}", env.evaluate_node(sum_id));

    block_on(run())
}

async fn run() -> Result<()> {
    pretty_env_logger::init();
    let e_loop = EventLoop::new();

    let win = WindowBuilder::new()
        .with_inner_size(LogicalSize::<u32>::from((720, 720)))
        .with_title("cancer soot")
        .build(&e_loop)?;

    debug!("Created window");

    let mut render = render::Render::new(&win).await?;

    let size = win.inner_size();
    let mut e_state = state::State::<easing::SinEasing>::new(size.width, size.height);

    let mut last_t = std::time::Instant::now();
    let mut last_fps = std::time::Instant::now();

    let mut deltas = Vec::new();

    let mut section_state = 0; // 0 = hidden, 1 = half, 2 = max

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
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Tab),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                match section_state {
                    0 => {
                        section_state = 1;
                        e_state.easing.set_goal(0.5);
                    }
                    1 => {
                        section_state = 2;
                        e_state.easing.set_goal(1.);
                    }
                    2 => {
                        section_state = 0;
                        e_state.easing.set_goal(0.);
                    }
                    _ => panic!("beans")
                }
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Caret),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                info!("State: {:?}", e_state);
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(k),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                info!("Pressed {:?}", k);
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
            deltas.push(dt.as_secs_f64());
            e_state.step(dt.as_secs_f64());
            last_t = now;

            if (now - last_fps) > std::time::Duration::from_secs(1) {
                last_fps = now;
                // unwrap is safe because we just pushed (assume no NaN or whatever)
                let max_delta = deltas.iter().cloned().fold(0., f64::max);
                let avg_delta = deltas.iter().cloned().sum::<f64>() / deltas.len() as f64;
                let min_delta = deltas.iter().cloned().fold(1./0., f64::min);
                deltas.clear();
                info!("FPS: {:.4}/{:.4}/{:.4}", 1.0/max_delta, 1.0/avg_delta, 1.0/min_delta);
            }


            match render.render(&e_state.get_render_state()) {
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
