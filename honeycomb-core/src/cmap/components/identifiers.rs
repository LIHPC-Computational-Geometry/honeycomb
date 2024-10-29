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

pub struct DartId(pub DartIdType);

impl_from_for_dartid!(VertexId);
impl_from_for_dartid!(EdgeId);
impl_from_for_dartid!(FaceId);
impl_from_for_dartid!(VolumeId);

// --- vertices

pub type VertexIdType = u32;

pub struct VertexId(pub VertexIdType);

impl_from_dartid!(VertexId);

// --- edges

pub type EdgeIdType = u32;

pub struct EdgeId(pub EdgeIdType);

impl_from_dartid!(EdgeId);

// --- faces

pub type FaceIdType = u32;

pub struct FaceId(pub FaceIdType);

impl_from_dartid!(FaceId);

// --- volumes

pub type VolumeIdType = u32;

pub struct VolumeId(pub VolumeIdType);

impl_from_dartid!(VolumeId);
