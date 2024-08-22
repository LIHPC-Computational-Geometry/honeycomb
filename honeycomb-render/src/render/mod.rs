pub mod camera;
mod picking;
mod scene;
mod update;

use bevy::prelude::*;
use bevy_mod_outline::OutlinePlugin;
use bevy_mod_picking::selection::SelectionPluginSettings;
use bevy_mod_picking::DefaultPickingPlugins;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        // camera
        app.add_systems(
            Update,
            camera::update_camera.run_if(camera::cursor_in_render),
        );
        // picking
        app.add_plugins(DefaultPickingPlugins.build())
            .add_plugins(OutlinePlugin)
            .insert_resource(SelectionPluginSettings::default())
            .add_systems(Update, picking::update_picking);
        // scene camera, light
        app.add_systems(Startup, scene::setup_scene);
    }
}
