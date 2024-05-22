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
    pub fn split_edge(&mut self, edge_id: EdgeIdentifier, midpoint_vertex: Option<T>) {
        if midpoint_vertex.is_some_and(|t| (t >= T::one()) | (t <= T::zero())) {
            println!("W: vertex placement for split is not in ]0;1[ -- result may be incoherent");
        }
        let d1 = edge_id as DartIdentifier;
        let d2 = self.beta::<2>(d1);
        // (*): unwrapping is ok because you probably shouldn't be using this method
        //      if both vertices of the edge aren't defined
        if d2 == NULL_DART_ID {
            let b1d1_old = self.beta::<1>(d1);
            let b1d1_new = self.add_free_dart();
            let v1 = self // (*)
                .vertex(self.vertex_id(d1))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            let v2 = self // (*)
                .vertex(self.vertex_id(b1d1_old))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            // unsew current dart
            self.one_unlink(d1);
            // rebuild the edge
            self.one_link(d1, b1d1_new);
            self.one_link(b1d1_new, b1d1_old);
            // insert the new vertex
            let seg = v2 - v1;
            self.insert_vertex(
                self.vertex_id(b1d1_new),
                midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
            );
        } else {
            let b1d1_old = self.beta::<1>(d1);
            let b1d2_old = self.beta::<1>(d2);
            let b1d1_new = self.add_free_darts(2);
            let b1d2_new = b1d1_new + 1;
            let v1 = self // (*)
                .vertex(self.vertex_id(d1))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            let v2 = self // (*)
                .vertex(self.vertex_id(d2))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            // unsew current darts
            self.one_unlink(d1);
            self.one_unlink(d2);
            self.two_unlink(d1);
            // rebuild the edge
            self.one_link(d1, b1d1_new);
            if b1d1_old != NULL_DART_ID {
                self.one_link(b1d1_new, b1d1_old);
            }
            self.one_link(d2, b1d2_new);
            if b1d2_old != NULL_DART_ID {
                self.one_link(b1d2_new, b1d2_old);
            }
            self.two_link(d1, b1d2_new);
            self.two_link(d2, b1d1_new);
            // insert the new vertex
            let seg = v2 - v1;
            self.insert_vertex(
                self.vertex_id(b1d1_new),
                midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
            );
        }
    }
}
