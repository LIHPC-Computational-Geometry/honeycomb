//! shader-compatible data structure code
//!
//! This module contains all code used to implement and describe data structures
//! that can be interpreted by the shader system.

// ------ IMPORTS

use bytemuck::{Pod, Zeroable};
use honeycomb_core::FaceIdentifier;

// ------ CONTENT

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Coords2Shader {
    position: [f32; 2],
    color: u32,
}

impl Coords2Shader {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32];

    pub fn new((x, y): (f32, f32), face_id: FaceIdentifier) -> Self {
        Self {
            position: [x, y],
            #[allow(clippy::unnecessary_cast)]
            color: face_id as u32,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Coords2Shader>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// pack of six => one arrow
pub const TEST_VERTICES: &[Coords2Shader] = &[
    Coords2Shader {
        position: [-0.5, 0.0],
        color: 0,
    },
    Coords2Shader {
        position: [0.5, -0.02],
        color: 0,
    },
    Coords2Shader {
        position: [0.5, 0.02],
        color: 0,
    },
    Coords2Shader {
        position: [0.5, -0.04],
        color: 0,
    },
    Coords2Shader {
        position: [0.5, 0.04],
        color: 0,
    },
    Coords2Shader {
        position: [0.6, 0.0],
        color: 0,
    },
];
