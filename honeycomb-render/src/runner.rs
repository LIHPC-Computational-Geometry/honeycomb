//! main execution code
//!
//! This module contains all code related to the main loop and execution setup.

// ------ IMPORTS

use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};

use crate::state::{App, State};
use crate::RenderParameters;
use honeycomb_core::{CMap2, CoordsFloat};

// ------ CONTENT

pub type MapRef<'a, T> = &'a CMap2<T>;

/// Main rendering structure
pub struct Runner {
    event_loop: EventLoop<()>,
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
        let mut app = App::new(render_params, map.unwrap());
        let _ = self.event_loop.run_app(&mut app);
    }

    /// UNIMPLEMENTED
    pub async fn run_async(&self) {
        unimplemented!()
    }
}

impl Default for Runner {
    fn default() -> Self {
        let event_loop = EventLoop::new().unwrap();

        Self { event_loop }
    }
}
