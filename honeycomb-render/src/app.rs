use crate::capture::{Capture, CaptureList};
use crate::plugins::{CapturePlugin, GuiPlugin, OptionsPlugin, ScenePlugin};
use bevy::prelude::App as BevyApp;
use bevy::prelude::*;
use honeycomb_core::{CMap2, CoordsFloat};

/// Default render structure.
///
/// This structure is essentially a wrapper around a regular `bevy` application. It only provides
/// an additional method ([`App::add_capture`]) to allow user to record map data that should be
/// displayed when running.
pub struct App {
    /// Inner `bevy` app.
    pub app: BevyApp,
    capture_list: CaptureList,
}

impl App {
    /// Add a [`Capture`] to the collection of the app. The capture is created on-the-fly using the
    /// specified reference to a combinatorial map.
    pub fn add_capture<T: CoordsFloat>(&mut self, cmap: &CMap2<T>) -> usize {
        let cap_id = self.capture_list.0.len();
        let capture = Capture::new(cap_id, cmap);
        self.capture_list.0.push(capture);
        cap_id
    }

    /// Launch the inner `bevy` app.
    pub fn run(mut self) {
        self.app.insert_resource(self.capture_list);
        self.app.run();
    }
}

impl Default for App {
    fn default() -> Self {
        let mut app = BevyApp::new();
        // resource
        app.insert_resource(Msaa::Sample4);
        // plugins
        app.add_plugins(DefaultPlugins)
            .add_plugins(OptionsPlugin)
            .add_plugins(GuiPlugin)
            .add_plugins(ScenePlugin)
            .add_plugins(CapturePlugin);
        Self {
            app,
            capture_list: CaptureList(Vec::new()),
        }
    }
}
