//! Graph colorign algorithms
//!
//! This module contains all coloring algorithms we implement for N-maps. The main purpose of such
//! algorithms is to help generate independent subsets of data for meshing algorithms to process.

use honeycomb_core::{
    attributes::AttrSparseVec,
    cmap::{OrbitPolicy, VertexIdType},
    prelude::{AttributeBind, AttributeUpdate},
};

mod dsatur;

pub use dsatur::color as color_dsatur;

// ---

/// Color attribute used to mark vertices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(u8);

impl AttributeUpdate for Color {
    fn merge(attr1: Self, _attr2: Self) -> Self {
        attr1
    }

    fn split(attr: Self) -> (Self, Self) {
        (attr, attr)
    }
}

impl AttributeBind for Color {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}

// ---

#[cfg(test)]
mod tests;
