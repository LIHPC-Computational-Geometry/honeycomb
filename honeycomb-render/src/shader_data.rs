//! shader-compatible data structure code
//!
//! This module contains all code used to implement and describe data structures
//! that can be interpreted by the shader system.

// ------ IMPORTS

use crate::representations::intermediates::Entity;
use bytemuck::{Pod, Zeroable};
use honeycomb_core::{CoordsFloat, FaceIdentifier, Vertex2};

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

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Coords2Shader>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

macro_rules! as_f32_array {
    ($coords: ident) => {
        [$coords.x().to_f32().unwrap(), $coords.y().to_f32().unwrap()]
    };
}

impl<T: CoordsFloat> From<(Vertex2<T>, Entity)> for Coords2Shader {
    fn from((v, e): (Vertex2<T>, Entity)) -> Self {
        Self {
            position: as_f32_array!(v),
            color: match e {
                Entity::Dart => 0,
                Entity::Face => 2,
            },
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
