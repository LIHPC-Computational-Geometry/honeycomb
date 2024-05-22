//! Advanced operations implementation
//!
//! This module contains code used to implement advanced operations, e.g. some non-standard,
//! higher-level abstractions that are useful in meshing algorithms.

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, EdgeIdentifier};

// ------ CONTENT

impl<T: CoordsFloat> CMap2<T> {
    /// Split an edge into to segments.
    ///
    /// <div class="warning">
    /// This implementation is 2D specific.
    /// </div>
    ///
    /// This method takes all darts of an edge and rebuild connections in order to create a new
    /// point on this edge. The position of the point default to the midway point, but it can
    /// optionally be specified.
    ///
    /// In order to minimize editing of embedded data, the original darts are kept to their
    /// original vertices while the new darts are used to model the new point.
    pub fn split_edge(&mut self, edge_id: EdgeIdentifier) {
        todo!()
    }
}
