//! # honeycomb
//!
//! Honeycomb aims to provide a safe, efficient and scalable implementation of combinatorial maps
//! for meshing applications. More specifically, the goal is to converge towards a (or multiple)
//! structure(s) adapted to algorithms exploiting GPU and many-core architectures.
//!
//! ## Structure
//!
//! This crate acts as the user-facing API, re-exporting components and items implemented in the
//! following sub-crates:
//!
//! - `honeycomb-core` -- core structures implementations
//! - `honeycomb-kernels` -- algorithm implementations
//! - `honeycomb-render` -- visual debugging tool
//!
//! ## Features
//!
//! Two features can be enabled to control which implementations are exposed:
//!
//! - `kernels` -- content from the `honeycomb-kernels` crate
//! - `render` -- content from the `honeycomb-render` crate
//!
//! Note that:
//! - the `kernels` feature is enabled by default since it requires minimal additional dependencies.
//! - the `render` feature is disabled by default; enabling it significantly lengthen the
//!   dependency tree as well as the compilation time.
//!
//! ## Quickstart
//!
//! For usage examples, refer to examples hosted in the [repository][EX]. Important items also have
//! example(s) included in their documentation:
//!
//! - [`CMap2`][honeycomb_core::cmap::CMap2]
//! - [`CMap3`][honeycomb_core::cmap::CMap3]
//! - [`CMapBuilder`][honeycomb_core::cmap::CMapBuilder]
//!
//! [EX]: https://github.com/LIHPC-Computational-Geometry/honeycomb/tree/master/examples

// --- enable doc_auto_cfg feature if compiling in nightly
#![allow(unexpected_cfgs)]
#![cfg_attr(nightly, feature(doc_auto_cfg))]

pub use honeycomb_core as core;
pub use honeycomb_core::stm;

#[cfg(feature = "kernels")]
pub use honeycomb_kernels as kernels;

#[cfg(feature = "render")]
pub use honeycomb_render as render;

/// commonly used items
///
/// This module contains all items commonly used to write a program using combinatorial maps.
/// These items are re-exported from their original crates for ease of use and should cover
/// all basic use cases.
pub mod prelude {
    // ------ CORE RE-EXPORTS

    pub use honeycomb_core::attributes::{
        AttrSparseVec, AttributeBind, AttributeError, AttributeUpdate,
    };
    pub use honeycomb_core::cmap::{
        BuilderError, CMap2, CMap3, CMapBuilder, DartIdType, EdgeIdType, FaceIdType,
        GridDescriptor, LinkError, NULL_DART_ID, NULL_EDGE_ID, NULL_FACE_ID, NULL_VERTEX_ID,
        NULL_VOLUME_ID, OrbitPolicy, SewError, VertexIdType, VolumeIdType,
    };
    pub use honeycomb_core::geometry::{
        CoordsError, CoordsFloat, Vector2, Vector3, Vertex2, Vertex3,
    };

    // ------ KERNELS RE-EXPORTS

    #[cfg(feature = "kernels")]
    pub use honeycomb_kernels::{cell_insertion, grisubal, remeshing, skewness, triangulation};

    // ------ RENDER RE-EXPORTS

    #[cfg(feature = "render")]
    pub use honeycomb_render::{render_2d_map, render_3d_map};
}
