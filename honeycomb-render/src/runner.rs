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

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub type MapRef<'a, const N_MARKS: usize, T> = &'static CMap2<N_MARKS, T>;
    } else {
        pub type MapRef<'a, const N_MARKS: usize, T> = &'a CMap2<N_MARKS, T>;
    }
}

async fn inner<const N_MARKS: usize, T: CoordsFloat>(
    event_loop: EventLoop<()>,
    window: Window,
    render_params: RenderParameters,
    map: Option<MapRef<'_, N_MARKS, T>>,
) {
    let mut state = if let Some(val) = map {
        State::new(&window, render_params, val).await
    } else {
        State::new_test(&window, render_params).await
    };

    event_loop
        .run(move |event, target| {
            match event {
                Event::WindowEvent {
                    window_id,
                    event: wevent,
                } => {
                    if window_id == state.window().id() && !state.input(&wevent) {
                        match wevent {
                            WindowEvent::Resized(new_size) => state.resize(Some(new_size)),
                            WindowEvent::RedrawRequested => {
                                state.update();
                                match state.render() {
                                    Ok(_) => {}
                                    Err(wgpu::SurfaceError::Lost) => state.resize(None),
                                    Err(wgpu::SurfaceError::OutOfMemory) => target.exit(), // kill if OOM
                                    Err(e) => eprintln!("{:?}", e),
                                }
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
    pub fn run<const N_MARKS: usize, T: CoordsFloat>(
        self,
        render_params: RenderParameters,
        map: Option<MapRef<'_, N_MARKS, T>>,
    ) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init().expect("could not initialize logger");
                wasm_bindgen_futures::spawn_local(inner(self.event_loop, self.window, render_params, map));
            } else {
                env_logger::init();
                pollster::block_on(inner(self.event_loop, self.window, render_params, map));
            }
        }
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
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowBuilderExtWebSys;
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("wasm-example")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            builder = builder.with_canvas(Some(canvas));
        }
        let window = builder.build(&event_loop).unwrap();

        Self { event_loop, window }
    }
}
