//! Advanced operations implementation
//!
//! This module contains code used to implement advanced operations, e.g. some non-standard,
//! higher-level abstractions that are useful in meshing algorithms.

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, DartIdentifier, EdgeIdentifier, Vertex2, NULL_DART_ID};

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
    ///
    /// For an illustration of both principles, refer to the example section.
    ///
    /// # Arguments
    ///
    /// - `edge_id: EdgeIdentifier` -- Edge to split in two.
    /// - `midpoint_vertex: Option<T>` -- Relative position of the new vertex, starting from the
    ///   vertex of the dart sharing `edge_id` as its identifier.
    ///
    /// # Panics
    ///
    /// This method may panic if the edge upon which the operation is performed does not have two
    /// defined vertices.
    ///
    /// # Example
    ///
    /// Given an edge made of darts `1` and `2`, these darts respectively encoding vertices
    /// `(0.0, 0.0)` and `(2.0, 0.0)`, calling `map.split_edge(1, Some(0.2))` would result in the
    /// creation of two new darts, a new vertex (ID `3`) of value `(0.4, 0.0)` and the following
    /// topology:
    ///
    /// ```text
    ///    +----1---->              +-1-> +-3->     |
    ///  1             2    =>    1      3      2   | + denote darts that encode vertex IDs
    ///    <----2----+              <-4-- <-2-+     |
    /// ```
    pub fn split_edge(&mut self, edge_id: EdgeIdentifier, midpoint_vertex: Option<T>) {
        if midpoint_vertex.is_some_and(|t| (t >= T::one()) | (t <= T::zero())) {
            println!("W: vertex placement for split is not in ]0;1[ -- result may be incoherent");
        }
        // base darts making up the edge
        let base_dart1 = edge_id as DartIdentifier;
        let base_dart2 = self.beta::<2>(base_dart1);
        // (*): unwrapping is ok since splitting an edge that does not have both its vertices
        //      defined is undefined behavior, therefore panic
        if base_dart2 == NULL_DART_ID {
            let b1d1_old = self.beta::<1>(base_dart1);
            let b1d1_new = self.add_free_dart();
            let v1 = self // (*)
                .vertex(self.vertex_id(base_dart1))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            let v2 = self // (*)
                .vertex(self.vertex_id(b1d1_old))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            // unsew current dart
            self.one_unlink(base_dart1);
            // rebuild the edge
            self.one_link(base_dart1, b1d1_new);
            self.one_link(b1d1_new, b1d1_old);
            // insert the new vertex
            let seg = v2 - v1;
            self.insert_vertex(
                self.vertex_id(b1d1_new),
                midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
            );
        } else {
            let b1d1_old = self.beta::<1>(base_dart1);
            let b1d2_old = self.beta::<1>(base_dart2);
            let b1d1_new = self.add_free_darts(2);
            let b1d2_new = b1d1_new + 1;
            let v1 = self // (*)
                .vertex(self.vertex_id(base_dart1))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            let v2 = self // (*)
                .vertex(self.vertex_id(base_dart2))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            // unsew current darts
            self.one_unlink(base_dart1);
            self.one_unlink(base_dart2);
            self.two_unlink(base_dart1);
            // rebuild the edge
            self.one_link(base_dart1, b1d1_new);
            if b1d1_old != NULL_DART_ID {
                self.one_link(b1d1_new, b1d1_old);
            }
            self.one_link(base_dart2, b1d2_new);
            if b1d2_old != NULL_DART_ID {
                self.one_link(b1d2_new, b1d2_old);
            }
            self.two_link(base_dart1, b1d2_new);
            self.two_link(base_dart2, b1d1_new);
            // insert the new vertex
            let seg = v2 - v1;
            self.insert_vertex(
                self.vertex_id(b1d1_new),
                midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
            );
        }
    }
}
