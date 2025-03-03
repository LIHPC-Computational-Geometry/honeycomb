use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::gui::{CustomTab, UiState};

/// Taken from the bevy
/// [cheatbook](https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html).
#[derive(Component)]
pub struct PanOrbitCamera {
    pub(crate) focus: Vec3,
    pub(crate) radius: f32,
    pub(crate) upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Camera update routine.
pub fn update_camera(
    window_q: Query<&Window>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    let window = window_q.single();
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Left;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.read() {
            pan += ev.delta * 2.;
        }
    }

    for ev in ev_scroll.read() {
        scroll += ev.y;

        scroll /= if cfg!(target_arch = "wasm32") {
            100.0
        } else {
            20.0
        };
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in &mut query {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }
        let window = vec2(
            window.physical_width() as f32,
            window.physical_height() as f32,
        );

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down { -delta } else { delta }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation *= pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;

            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    ev_motion.clear();
}

#[allow(clippy::missing_panics_doc)]
/// Detects if the cursor is positioned in the render tab.
///
/// This is used to ignore camera related input when interacting with something other than the
pub fn cursor_in_render(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    ui_state: Res<UiState>,
) -> bool {
    // returns true if and only if the cursor is positioned in the render tab
    if let Some(position) = q_windows.single().cursor_position() {
        let tree = ui_state.tree();
        if let Some((node_idx, _)) = tree.find_tab(&CustomTab::Render) {
            let rect = &tree[node_idx].rect().expect("unreachable");
            return rect.contains(position.to_array().into());
        }
        return false;
    }
    false
}
