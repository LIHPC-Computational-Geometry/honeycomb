use crate::components::PanOrbitCamera;
use bevy::prelude::*;

macro_rules! spawn_plight {
    ($cmd: ident, $x: expr, $y: expr, $z: expr) => {
        $cmd.spawn(PointLightBundle {
            point_light: PointLight {
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz($x, $y, $z),
            ..default()
        });
    };
}

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

    spawn_plight!(commands, 0.0, 0.0, 6.0);
    spawn_plight!(commands, 10.0, 0.0, 4.0);
    spawn_plight!(commands, 0.0, 10.0, 4.0);
    spawn_plight!(commands, -10.0, 0.0, 4.0);
    spawn_plight!(commands, 0.0, -10.0, 4.0);
}
