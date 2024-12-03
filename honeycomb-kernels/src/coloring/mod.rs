use honeycomb_core::{
    attributes::AttrSparseVec,
    cmap::{OrbitPolicy, VertexIdType},
    prelude::{AttributeBind, AttributeUpdate},
};

mod dsatur;

pub use dsatur::color as color_dsatur;

// --- common content

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
