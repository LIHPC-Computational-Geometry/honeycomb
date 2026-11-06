use honeycomb_core::{
    attributes::{AttrSparseVec, AttributeBind, AttributeError, AttributeUpdate},
    cmap::{OrbitPolicy, VertexIdType},
};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Weight(pub u32);

impl AttributeUpdate for Weight {
    // when merging two weights, we add them
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        Ok(Self(attr1.0 + attr2.0))
    }

    // when splitting, we do an approximate 50/50
    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        // adding the % to keep things conservative
        Ok((Self(attr.0 / 2 + attr.0 % 2), Self(attr.0 / 2)))
    }

    // if we have to merge from a single value, we assume the "other" is 0
    fn merge_incomplete(attr: Self) -> Result<Self, AttributeError> {
        Ok(attr)
    }
}

impl AttributeBind for Weight {
    // Weight values will be stored in an `AttrSparseVec`
    type StorageType = AttrSparseVec<Self>;
    // Weights bind to vertices
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}
