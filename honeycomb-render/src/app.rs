use bevy::prelude::App as BevyApp;
use bevy::prelude::*;
use honeycomb_core::{CMap2, CoordsFloat};

pub struct App {
    app: BevyApp,
}

impl App {
    pub fn add_capture<T: CoordsFloat>(&mut self, cmap: &CMap2<T>) {
        todo!()
    }
}

impl Default for App {
    fn default() -> Self {
        let mut app = BevyApp::new();
        // resource
        app.insert_resource(Msaa::Sample4);
        // plugins
        app.add_plugins(DefaultPlugins);
        Self { app }
    }
}
