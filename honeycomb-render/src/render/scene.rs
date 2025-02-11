use bevy::prelude::*;

use crate::components::PanOrbitCamera;

/// Scene setup routine.
pub fn setup_scene(mut commands: Commands) {
    let camera_transform = Transform::from_xyz(0.0, 0.0, 5.0);

    commands.spawn((
        PanOrbitCamera {
            radius: camera_transform.translation.length(),
            ..Default::default()
        },
        Camera3dBundle {
            transform: camera_transform.looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));
}
