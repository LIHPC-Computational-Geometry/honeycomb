use crate::GuiPlugin;
use crate::OptionsPlugin;
use crate::ScenePlugin;
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

    pub fn run(&mut self) {
        let _ = self.app.run();
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
            .add_plugins(ScenePlugin);
        Self { app }
    }
}
