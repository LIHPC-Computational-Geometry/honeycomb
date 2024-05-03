//! main execution code
//!
//! This module contains all code related to the main loop and execution setup.

// ------ IMPORTS

use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::state::State;
use crate::RenderParameters;
use honeycomb_core::{CMap2, CoordsFloat};

// ------ CONTENT

pub type MapRef<'a, T> = &'a CMap2<T>;

const TARGET_FPS: f32 = 240.;

/// This yields an approximate 240 FPS
const MS_PER_FRAME: u128 = (1000. / TARGET_FPS) as u128;

async fn inner<T: CoordsFloat>(
    event_loop: EventLoop<()>,
    window: Window,
    render_params: RenderParameters,
    map: Option<MapRef<'_, T>>,
) {
    let mut state = if let Some(val) = map {
        State::new(&window, render_params, val).await
    } else {
        State::new_test(&window, render_params).await
    };
    println!("I: Press F1 to quit");
    event_loop
        .run(move |event, target| {
            // process events
            match event {
                Event::WindowEvent {
                    window_id,
                    event: wevent,
                } => {
                    if window_id == state.window().id() && !state.input(&wevent, target) {
                        match wevent {
                            WindowEvent::Resized(new_size) => state.resize(Some(new_size)),
                            WindowEvent::RedrawRequested => {
                                let start = std::time::Instant::now();
                                state.update();
                                match state.render() {
                                    Ok(_) => {}
                                    Err(wgpu::SurfaceError::Lost) => state.resize(None),
                                    Err(wgpu::SurfaceError::OutOfMemory) => target.exit(), // kill if OOM
                                    Err(e) => eprintln!("{:?}", e),
                                };
                                // put a hard cap on the rendering speed
                                std::thread::sleep(std::time::Duration::from_millis(
                                    (MS_PER_FRAME - start.elapsed().as_millis()) as u64,
                                ));
                            }
                            WindowEvent::CloseRequested => target.exit(),
                            _ => {}
                        };
                    }
                }
                Event::AboutToWait => {
                    state.window().request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}

/// Main rendering structure
pub struct Runner {
    event_loop: EventLoop<()>,
    window: Window,
}

impl Runner {
    /// Main rendering method.
    ///
    /// # Arguments
    ///
    /// - `render_params: RenderParameters` -- Render parameterization.
    /// - `map: Option<&CMap2>` -- Optionnal reference to the map that should be rendered
    ///
    /// If no reference is passed to the method, a hardcoded example will be rendered instead.
    ///
    /// # Example
    ///
    /// Because the renderer uses the core and utils crates, examples are provided as standalone
    /// files rather than in the doc. You can run them using the following command:
    ///
    /// ```shell
    /// cargo run --example <EXAMPLE>
    /// ```
    ///
    pub fn run<T: CoordsFloat>(self, render_params: RenderParameters, map: Option<MapRef<'_, T>>) {
        env_logger::init();
        pollster::block_on(inner(self.event_loop, self.window, render_params, map));
    }

    /// UNIMPLEMENTED
    pub async fn run_async(&self) {
        unimplemented!()
    }
}

impl Default for Runner {
    fn default() -> Self {
        let event_loop = EventLoop::new().unwrap();
        #[allow(unused_mut)]
        let mut builder = winit::window::WindowBuilder::new();
        let window = builder.build(&event_loop).unwrap();

        Self { event_loop, window }
    }
}
