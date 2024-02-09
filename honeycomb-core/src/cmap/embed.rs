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

#[derive(Clone, Copy, Debug)]
/// Dart-cell associative structure
///
/// Structure used to store the associated vertex, face and volume
/// identifiers to a dart. The structure technically contains only
/// cell identifiers, the association with a dart ID is done implicitly
/// through storage indexing.
///
/// Each field is kept public as editing operations can happen during
/// execution (e.g. a sewing operation will "fuse" some geometric
/// objects).
///
/// # Example
///
/// ```
/// use honeycomb_core::{cmap::embed::DartCells, Dart};
///
/// let darts = vec![Dart::NULL, Dart::from(1)];
/// let embedded_cells = vec![DartCells::NULL; 2];
///
/// println!("dart {} associated cells: {:#?}", darts[1].id(), embedded_cells[darts[1].id()]);
/// ```
///
pub struct DartCells {
    /// Vertex unique identifier.
    pub vertex_id: usize,
    /// Face unique identifier.
    pub face_id: usize,
    /// Volume unique identifier.
    pub volume_id: usize,
}

impl DartCells {
    /// Null value for the structure. This technically should not be used
    /// since acessing the null dart means stopping whatever operation
    /// was happening.
    pub const NULL: DartCells = Self {
        vertex_id: 0,
        face_id: 0,
        volume_id: 0,
    };
}

/// Type definition for 3D vertices representation.
///
/// This type can also be used for 2D spaces by fixing one of the three
/// coordinates.
pub type Vertex = [f64; 3];

/// Edge object
///
/// An edge is composed of two vertices. Those are stored using their
/// indices in order to avoid duplicating floating-point numbers. If
/// needed, the structure can be interepreted as oriented.
///
/// # Example
///
/// See [Face] example.
///
pub struct Edge {
    /// First vertex.
    pub v1: usize,
    /// Second vertex.
    pub v2: usize,
}

/// Face object
///
/// A face is made up of a varying number of edges (e.g. 3 for a triangle).
/// The faces are stored using indices to avoid duplicating floating-point
/// numbers.
///
/// NOTE: It may be possible to replace the Vec with an upper-bound structure
/// to limit heap allocation during execution. We could also add references to
/// the vertices and edge list inside the structure?
///
/// # Example
///
/// This code corresponds to the initialization of 4 vertices, 5 edges, used
/// to build 2 faces: a square and a triangle, both aligned on the origin.
///
/// ```
/// use honeycomb_core::cmap::embed::{Vertex, Edge, Face};
///
/// let vertices = [
///     [0.0, 0.0, 0.0],
///     [1.0, 0.0, 0.0],
///     [1.0, 1.0, 0.0],
///     [0.0, 1.0, 0.0],
/// ];
///
/// let edges = [
///     Edge { v1: 0, v2: 1 },
///     Edge { v1: 1, v2: 2 },
///     Edge { v1: 2, v2: 3 },
///     Edge { v1: 3, v2: 0 },
///     Edge { v1: 2, v2: 0 },
/// ];
///
/// let square_face = Face { edges: vec![0, 1, 2, 3] };
/// let triangle_face = Face { edges: vec![0, 1, 4] };
/// ```
///
pub struct Face {
    /// List of all edges composing the face.
    pub edges: Vec<usize>,
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
