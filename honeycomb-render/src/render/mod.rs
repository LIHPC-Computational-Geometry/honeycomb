pub mod camera;
pub mod picking;
pub mod scene;
#[allow(clippy::needless_pass_by_value)]
pub mod update;

use crate::capture::FocusedCapture;
use crate::resources::{
    DartHeadMul, DartRenderColor, DartShrink, DartWidth, EdgeRenderColor, EdgeWidth,
    VertexRenderColor, VertexWidth,
};
use bevy::prelude::*;
use bevy_mod_outline::OutlinePlugin;
use bevy_mod_picking::selection::SelectionPluginSettings;
use bevy_mod_picking::DefaultPickingPlugins;

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
                        .and_then(not(resource_added::<FocusedCapture>)),
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
                    .or_else(
                        resource_changed::<DartHeadMul>
                            .and_then(not(resource_added::<DartHeadMul>)),
                    ),
            ),
        );
        app.add_systems(
            Update,
            update::dart_bodies_handles.run_if(
                resource_changed::<DartWidth>
                    .and_then(not(resource_added::<DartWidth>))
                    .or_else(
                        resource_changed::<DartShrink>.and_then(not(resource_added::<DartShrink>)),
                    ),
            ),
        );
        app.add_systems(
            Update,
            (update::dart_heads_transform, update::dart_bodies_transform)
                .run_if(resource_changed::<DartShrink>.and_then(not(resource_added::<DartShrink>))),
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
                resource_changed::<VertexWidth>.and_then(not(resource_added::<VertexWidth>)),
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
            update::edges_handle
                .run_if(resource_changed::<EdgeWidth>.and_then(not(resource_added::<EdgeWidth>))),
        );
    }
}
