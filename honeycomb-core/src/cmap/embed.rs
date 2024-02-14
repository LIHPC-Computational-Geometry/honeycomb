//! Geometric data embedding
//!
//! The initialization of geometric data should eventually be done by
//! the combinatorial map constructor. Until then, the user has to
//! manually initialize it through this module's tools.
//!
//! This module contains all code used to handle geometric data
//! embedding. This includes spatial position as well as i-cells'
//! identifiers.

// ------ MODULE DECLARATIONS

// ------ IMPORTS

// ------ CONTENT

/// Type definition for vertex identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type VertexIdentifier = usize;

/// Type definition for face identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type FaceIdentifier = usize;

/// Type definition for volume identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type VolumeIdentifier = usize;

#[derive(Debug, Default)]
pub enum SewPolicy {
    #[default]
    StretchLeft,
    StretchRight,
    StretchAverage,
}

#[derive(Debug, Default)]
pub enum UnsewPolicy {
    #[default]
    Duplicate,
}

/// Type definition for 2D vertices representation.
pub type Vertex2 = [f64; 2];

/// Type definition for 3D vertices representation.
pub type Vertex3 = [f64; 3];

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
/// use honeycomb_core::cmap::embed::{Vertex2, Face};
///
/// let vertices = [
///     [0.0, 0.0],
///     [1.0, 0.0],
///     [1.0, 1.0],
///     [0.0, 1.0],
///     [2.0, 0.0],
/// ];
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
