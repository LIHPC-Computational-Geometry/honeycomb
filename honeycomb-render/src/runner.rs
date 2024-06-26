//! main execution code
//!
//! This module contains all code related to the main loop and execution setup.

// ------ IMPORTS

use winit::event_loop::EventLoop;

use crate::{state::App, MapRef, RenderParameters};
use honeycomb_core::CoordsFloat;

// ------ CONTENT

#[allow(clippy::missing_panics_doc)]
/// Main rendering function.
///
/// # Arguments
///
/// - `render_params: RenderParameters` -- Render parameterization.
/// - `map: Option<&CMap2>` -- Optional reference to the map that should be rendered
///
/// If no reference is passed to the method, a hardcoded example will be rendered instead.
///
/// # Example
///
/// Because the renderer uses the core and utils crates, examples are provided as standalone
/// files rather than in the doc. You can find them in the project's repository, under the
/// `honeycomb-examples` crate and run them using `cargo`:
///
/// ```shell
/// cargo run --example <EXAMPLE>
/// ```
///
pub fn launch<T: CoordsFloat>(render_params: RenderParameters, map: Option<MapRef<'_, T>>) {
    // enable custom env logging
    env_logger::init();
    // build app & event loop
    let mut app = App::new(render_params, map);
    let event_loop = EventLoop::new().unwrap();
    let _ = event_loop.run_app(&mut app);
}

/// UNIMPLEMENTED
pub async fn launch_async<T: CoordsFloat>(
    _render_params: RenderParameters,
    _map: Option<MapRef<'_, T>>,
) {
    unimplemented!()
}
