// ------ IMPORTS

// ------ CONTENT

use std::fmt::Display;

macro_rules! impl_from_dartid {
    ($idty: ty) => {
        impl From<DartId> for $idty {
            fn from(dart_id: DartId) -> Self {
                Self(dart_id.0.into())
            }
        }
    };
}

macro_rules! impl_from_for_dartid {
    ($idty: ty) => {
        impl From<$idty> for DartId {
            fn from(id: $idty) -> Self {
                Self(id.0.into())
            }
        }
    };
}

macro_rules! impl_display {
    ($idty: ty, $fmt: expr) => {
        impl Display for $idty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $fmt, self.0)
            }
        }
    };
}

// --- darts

/// Dart ID representation type.
pub type DartIdType = u32;

/// Strongly-typed dart ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DartId(pub DartIdType);

/// Null dart ID value.
pub const NULL_DART_ID: DartId = DartId(0);

impl_from_for_dartid!(VertexId);
impl_from_for_dartid!(EdgeId);
impl_from_for_dartid!(FaceId);
impl_from_for_dartid!(VolumeId);
impl_display!(DartId, "d{}");

// --- vertices

/// Vertex ID representation type.
pub type VertexIdType = u32;

/// Strongly-typed vertex ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VertexId(pub VertexIdType);

/// Null vertex ID value.
pub const NULL_VERTEX_ID: VertexId = VertexId(0);

impl_from_dartid!(VertexId);

// --- edges

/// Edge ID representation type.
pub type EdgeIdType = u32;

/// Strongly-typed edge ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeId(pub EdgeIdType);

/// Null edge ID value.
pub const NULL_EDGE_ID: EdgeId = EdgeId(0);

impl_from_dartid!(EdgeId);

// --- faces

/// Face ID representation type.
pub type FaceIdType = u32;

/// Strongly-typed face ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FaceId(pub FaceIdType);

/// Null face ID value.
pub const NULL_FACE_ID: FaceId = FaceId(0);

impl_from_dartid!(FaceId);

// --- volumes

/// Volume ID representation type.
pub type VolumeIdType = u32;

/// Strongly-typed volume ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VolumeId(pub VolumeIdType);

/// Null volume ID value.
pub const NULL_VOLUME_ID: VolumeId = VolumeId(0);

impl_from_dartid!(VolumeId);
