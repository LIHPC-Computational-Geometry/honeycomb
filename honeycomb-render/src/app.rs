use crate::capture::{Capture, CaptureList};
use crate::plugins::*;
use bevy::prelude::App as BevyApp;
use bevy::prelude::*;
use honeycomb_core::{CMap2, CoordsFloat};

pub struct App {
    pub app: BevyApp,
    capture_list: CaptureList,
}

impl App {
    pub fn add_capture<T: CoordsFloat>(mut self, cmap: &CMap2<T>) -> Self {
        let cap_id = self.capture_list.0.len();
        let capture = Capture::new(cap_id, cmap);
        self.capture_list.0.push(capture);
        self
    }

    pub fn run(mut self) {
        self.app.insert_resource(self.capture_list);
        // .add_systems(populate_entities.run_if(resource_added::<CaptureList>)); or smth
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
