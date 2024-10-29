// ------ IMPORTS

// ------ CONTENT

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

// --- darts

pub type DartIdType = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DartId(pub DartIdType);

const NULL_DART_ID: DartId = DartId(0);

impl_from_for_dartid!(VertexId);
impl_from_for_dartid!(EdgeId);
impl_from_for_dartid!(FaceId);
impl_from_for_dartid!(VolumeId);

// --- vertices

pub type VertexIdType = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexId(pub VertexIdType);

const NULL_VERTEX_ID: VertexId = VertexId(0);

impl_from_dartid!(VertexId);

// --- edges

pub type EdgeIdType = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EdgeId(pub EdgeIdType);

const NULL_EDGE_ID: EdgeId = EdgeId(0);

impl_from_dartid!(EdgeId);

// --- faces

pub type FaceIdType = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FaceId(pub FaceIdType);

const NULL_FACE_ID: FaceId = FaceId(0);

impl_from_dartid!(FaceId);

// --- volumes

pub type VolumeIdType = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VolumeId(pub VolumeIdType);

const NULL_VOLUME_ID: VolumeId = VolumeId(0);

impl_from_dartid!(VolumeId);
