//! camera framework code
//!
//! This module contains all code used to model the camera system of the renderer.

// ------ IMPORTS

use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{Key, NamedKey},
};

// ------ CONTENT

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub const SPEED_FACTOR: f32 = 0.005;
    } else {
        pub const SPEED_FACTOR: f32 = 0.05;
    }
}

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

impl CameraUniform {
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state, logical_key, ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match logical_key {
                    Key::Named(named) => match named {
                        NamedKey::ArrowUp => {
                            self.is_up_pressed = is_pressed;
                            true
                        }
                        NamedKey::ArrowLeft => {
                            self.is_left_pressed = is_pressed;
                            true
                        }
                        NamedKey::ArrowDown => {
                            self.is_down_pressed = is_pressed;
                            true
                        }
                        NamedKey::ArrowRight => {
                            self.is_right_pressed = is_pressed;
                            true
                        }
                        _ => false,
                    },
                    Key::Character(smolstr) => {
                        if smolstr.as_str() == "f" {
                            self.is_forward_pressed = is_pressed;
                            true
                        } else if smolstr.as_str() == "b" {
                            self.is_backward_pressed = is_pressed;
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_dir = forward.normalize();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed
            && camera.eye.z + forward_dir.z * SPEED_FACTOR * camera.eye.z > 2.0 * camera.znear
        {
            self.speed = SPEED_FACTOR * camera.eye.z;
            camera.eye += forward_dir * self.speed;
        }
        if self.is_backward_pressed
            && camera.eye.z - forward_dir.z * SPEED_FACTOR * camera.eye.z < 2.0 * camera.zfar
        {
            self.speed = SPEED_FACTOR * camera.eye.z;
            camera.eye -= forward_dir * self.speed;
        }

        let right = forward_dir.cross(camera.up);

        if self.is_right_pressed {
            camera.eye += right * self.speed;
            camera.target += right * self.speed;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed;
            camera.target -= right * self.speed;
        }
        if self.is_up_pressed {
            camera.eye += camera.up * self.speed;
            camera.target += camera.up * self.speed;
        }
        if self.is_down_pressed {
            camera.eye -= camera.up * self.speed;
            camera.target -= camera.up * self.speed;
        }
    }
}
