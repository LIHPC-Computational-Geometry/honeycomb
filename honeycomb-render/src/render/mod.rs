pub mod camera;
pub mod picking;
pub mod scene;
pub mod update;

use bevy::input::common_conditions::input_just_released;
use bevy::prelude::*;
use bevy_mod_outline::OutlinePlugin;
use bevy_mod_picking::selection::SelectionPluginSettings;
use bevy_mod_picking::DefaultPickingPlugins;

use crate::capture::FocusedCapture;
use crate::resources::{
    DartHeadMul, DartRenderColor, DartShrink, DartWidth, EdgeRenderColor, EdgeWidth,
    VertexRenderColor, VertexWidth,
};

/// Plugin handling scene setup and updates.
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        // camera
        app.add_systems(Startup, scene::setup_scene).add_systems(
            Update,
            camera::update_camera.run_if(camera::cursor_in_render),
        );

        // picking
        app.add_plugins(DefaultPickingPlugins.build())
            .add_plugins(OutlinePlugin)
            .insert_resource(SelectionPluginSettings::default())
            .add_systems(Update, picking::update_picking);

        // update displayed content

        // FocusedCapture change
        app.add_systems(
            Update,
            (
                update::dart_render,
                update::vertices_render,
                update::edges_render,
            )
                .run_if(
                    resource_changed::<FocusedCapture>
                        .and_then(not(resource_added::<FocusedCapture>))
                        .and_then(input_just_released(MouseButton::Left)),
                ),
        );

        // dart updates
        app.add_systems(
            Update,
            (update::dart_render, update::dart_mat_handle).run_if(
                resource_changed::<DartRenderColor>
                    .and_then(not(resource_added::<DartRenderColor>)),
            ),
        );
        app.add_systems(
            Update,
            update::dart_heads_handle.run_if(
                resource_changed::<DartWidth>
                    .and_then(not(resource_added::<DartWidth>))
                    .and_then(input_just_released(MouseButton::Left))
                    .or_else(
                        resource_changed::<DartHeadMul>
                            .and_then(not(resource_added::<DartHeadMul>))
                            .and_then(input_just_released(MouseButton::Left)),
                    ),
            ),
        );
        app.add_systems(
            Update,
            update::dart_bodies_handles.run_if(
                resource_changed::<DartWidth>
                    .and_then(not(resource_added::<DartWidth>))
                    .and_then(input_just_released(MouseButton::Left))
                    .or_else(
                        resource_changed::<DartShrink>
                            .and_then(not(resource_added::<DartShrink>))
                            .and_then(input_just_released(MouseButton::Left)),
                    ),
            ),
        );
        app.add_systems(
            Update,
            (update::dart_heads_transform, update::dart_bodies_transform).run_if(
                resource_changed::<DartShrink>
                    .and_then(not(resource_added::<DartShrink>))
                    .and_then(input_just_released(MouseButton::Left)),
            ),
        );

        // vertex updates
        app.add_systems(
            Update,
            (update::vertices_render, update::vertices_mat_handle).run_if(
                resource_changed::<VertexRenderColor>
                    .and_then(not(resource_added::<VertexRenderColor>)),
            ),
        );
        app.add_systems(
            Update,
            update::vertices_handle.run_if(
                resource_changed::<VertexWidth>
                    .and_then(not(resource_added::<VertexWidth>))
                    .and_then(input_just_released(MouseButton::Left)),
            ),
        );
        // edge updates
        app.add_systems(
            Update,
            (update::edges_render, update::edges_mat_handle).run_if(
                resource_changed::<EdgeRenderColor>
                    .and_then(not(resource_added::<EdgeRenderColor>)),
            ),
        );
        app.add_systems(
            Update,
            update::edges_handle.run_if(
                resource_changed::<EdgeWidth>
                    .and_then(not(resource_added::<EdgeWidth>))
                    .and_then(input_just_released(MouseButton::Left)),
            ),
        );
    }
}
