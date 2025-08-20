//! collection of utilities for meshing algorithms

mod anchors;
mod routines;

pub use anchors::{
    BodyIdType, CurveIdType, EdgeAnchor, FaceAnchor, NodeIdType, SurfaceIdType, VertexAnchor,
};
pub use routines::{
    compute_tet_orientation, is_orbit_orientation_consistent, locate_containing_tet,
};
