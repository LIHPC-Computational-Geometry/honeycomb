//! Geometric data embedding
//!
//! Note:
//! - The initialization of geometric data should eventually be done by
//!   the combinatorial map constructor. Until then, the user has to
//!   manually initialize it through this module's tools.
//! - A custom vector type is needed
//!
//! This module contains all code used to handle geometric data
//! embedding. This includes ID types, spatial representation,
//! (un)sewing policies.

// ------ MODULE DECLARATIONS

// ------ IMPORTS

#[cfg(doc)]
use crate::CMap2;

// ------ CONTENT

/// Type definition for vertex identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type VertexIdentifier = u32;

/// Type definition for face identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type FaceIdentifier = u32;

/// Type definition for volume identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type VolumeIdentifier = u32;

/// Geometrical policy of the sewing operation.
///
/// All sewing operation include two darts, which we refer to as
/// *lhs_dart* and *rhs_dart*, hence the naming conventions.
///
/// For more information on why a strict policy definition is needed,
/// refer to the [user guide][UG].
///
/// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
///
/// # Example
///
/// Refer to the user guide and [CMap2] examples.
///
#[derive(Debug, Default)]
pub enum SewPolicy {
    /// Default policy.
    ///
    /// *lhs_dart* embedded data is used to overwrite *rhs_dart*
    /// embedded data, effectively stretching *rhs_dart* to
    /// *lhs_dart*'s position.
    #[default]
    StretchLeft,
    /// *rhs_dart* embedded data is used to overwrite *lhs_dart*
    /// embedded data, effectively stretching *lhs_dart* to
    /// *rhs_dart*'s position.
    StretchRight,
    /// *lhs_dart* & *rhs_dart* embedded data are both used to
    /// compute a "middle ground" used to overwrite both initial
    /// positions.
    StretchAverage,
}

/// Geometrical policy of the sewing operation.
///
/// All unsewing operation include two darts, which we refer to as
/// *lhs_dart* and *rhs_dart*, hence the naming conventions. Note that
/// the second dart is not necessarily specified as it can be computed
/// the index of the beta function affected by the operation.
///
/// For more information on why a strict policy definition is needed,
/// refer to the [user guide][UG].
///
/// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
///
/// # Example
///
/// Refer to the user guide and [CMap2] examples.
///
#[derive(Debug, Default)]
pub enum UnsewPolicy {
    /// Default policy.
    ///
    /// Vertices associated with the unsewed darts are duplicated in order
    /// to allow for a shift in coordinates without mutual stretching.
    #[default]
    Duplicate,
    DoNothing,
}

#[derive(Clone, Debug, Default)]
/// Face object
///
/// A face is made up of a varying number of corners (e.g. 3 for a triangle).
/// The corners are stored in specific order to model the connections forming
/// the face; Additionally, a boolean indicates whether there is a connection
/// between the last corner and the first, effectively closing the face.
///
/// NOTE: It may be possible to replace the Vec with an upper-bound structure
/// to limit heap allocation during execution. We could also add references to
/// the vertices and edge list inside the structure?
///
/// # Example
///
/// This code corresponds to the initialization of 4 vertices used to build
/// 2 faces: a square and a triangle.
///
/// ```
/// use honeycomb_core::{Vertex2, Face};
///
/// let vertices = [
///     [0.0, 0.0],
///     [1.0, 0.0],
///     [1.0, 1.0],
///     [0.0, 1.0],
///     [2.0, 0.0],
/// ].map(Vertex2::from);
///
/// let square_face = Face { corners: vec![0, 1, 2, 3], closed: true };
/// let triangle_face = Face { corners: vec![1, 4, 2], closed: true };
///
/// ```
///
/// This corresponds to the following figure:
///
/// ```text
///
/// 1.0  +------+\_
///      |      |  \_
///      |      |    \_
///      |      |      \
/// 0.0  +------+------+
///      0.0    1.0    2.0
///
/// ```
pub struct Face {
    /// Ordered list of all corners composing the face.
    pub corners: Vec<VertexIdentifier>,
    /// Boolean indicating whether there is a connection between
    /// `self.corners.last()` and `self.corners.first`.
    pub closed: bool,
}

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
