//! Advanced operations implementation
//!
//! This module contains code used to implement advanced operations, e.g. some non-standard,
//! higher-level abstractions that are useful in meshing algorithms.

// ------ IMPORTS

use crate::geometry::CoordsFloat;
use crate::prelude::{CMap2, DartIdentifier, EdgeIdentifier, Vertex2, NULL_DART_ID};

// ------ CONTENT

/// **Advanced operations: edge splitting**
impl<T: CoordsFloat> CMap2<T> {
    /// Split an edge into two segments.
    ///
    /// <div class="warning">
    /// This implementation is 2D specific.
    /// </div>
    ///
    /// This method takes all darts of an edge and rebuild connections in order to create a new
    /// point on this edge. The position of the point defaults to the midway point, but it can
    /// optionally be specified.
    ///
    /// In order to minimize editing of embedded data, the original darts are kept to their
    /// original vertices while the new darts are used to model the new point.
    ///
    /// For an illustration of both principles, refer to the example.
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
    /// creation of two new darts, a new vertex (ID `3`) at position `(0.4, 0.0)` and the following
    /// topology:
    ///
    /// ```text
    ///    +----1---->              +-1-> +-3->     |
    ///  1             2    =>    1      3      2   | + denote darts that encode vertex IDs
    ///    <----2----+              <-4-- <-2-+     |
    /// ```
    pub fn split_edge(
        &mut self,
        edge_id: EdgeIdentifier,
        new_darts: (DartIdentifier, DartIdentifier), // 2D => statically known number of darts
        midpoint_vertex: Option<T>,
    ) {
        // midpoint check
        if midpoint_vertex.is_some_and(|t| (t >= T::one()) | (t <= T::zero())) {
            println!("W: vertex placement for split is not in ]0;1[ -- result may be incoherent");
        }
        // new darts (minimal) check,
        let (b1d1_new, b1d2_new) = new_darts;
        if new_darts.0 == NULL_DART_ID || !self.is_free(new_darts.0) {
            println!("W: dart {b1d1_new} cannot be used in split_edge -- passed darts should be non-null and free");
            println!("   skipping split...");
            return;
        }

        // base darts making up the edge
        let base_dart1 = edge_id as DartIdentifier;
        let base_dart2 = self.beta::<2>(base_dart1);
        // (*): unwrapping is ok since splitting an edge that does not have both its vertices
        //      defined is undefined behavior, therefore panic
        if base_dart2 == NULL_DART_ID {
            let b1d1_old = self.beta::<1>(base_dart1);
            let v1 = self // (*)
                .vertex(self.vertex_id(base_dart1))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            let v2 = self // (*)
                .vertex(self.vertex_id(b1d1_old))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            // unsew current dart
            // self.one_unlink(base_dart1);
            self.betas[base_dart1 as usize][1] = 0;
            self.betas[b1d1_old as usize][0] = 0;
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
            // check the second new dart
            if b1d2_new == NULL_DART_ID || !self.is_free(b1d2_new) {
                println!("W: dart {b1d2_new} cannot be used in split_edge -- passed darts should be non-null and free");
                println!("   skipping split...");
                return;
            }

            let b1d1_old = self.beta::<1>(base_dart1);
            let b1d2_old = self.beta::<1>(base_dart2);
            let v1 = self // (*)
                .vertex(self.vertex_id(base_dart1))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            let v2 = self // (*)
                .vertex(self.vertex_id(base_dart2))
                .expect("E: attempt to split an edge that is not fully defined in the first place");
            // unsew current darts
            // self.one_unlink(base_dart1);
            self.betas[base_dart1 as usize][1] = 0;
            self.betas[b1d1_old as usize][0] = 0;
            // self.one_unlink(base_dart2);
            self.betas[base_dart2 as usize][1] = 0;
            self.betas[b1d2_old as usize][0] = 0;
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

    /// Split an edge into `n` segments.
    ///
    /// <div class="warning">
    /// This implementation is 2D specific.
    /// </div>
    ///
    /// # Arguments
    ///
    /// - `edge_id: EdgeIdentifier` -- Edge to split in two.
    /// - `midpoint_vertices: I` -- Relative positions of new vertices, starting from the
    ///   vertex of the dart sharing `edge_id` as its identifier.
    ///
    /// ## Generics
    ///
    /// - `I: Iterator<Item = T>` -- Iterator over `T` values. These should be in the `]0; 1[` open range.
    ///
    /// # Panics
    ///
    /// This method may panic if the edge upon which the operation is performed does not have two defined vertices.
    ///
    /// # Example
    ///
    /// ```
    /// # use honeycomb_core::prelude::{CMap2, CMapBuilder, NULL_DART_ID, Vertex2};
    /// // before
    /// //    <--2---
    /// //  1         2
    /// //    ---1-->
    /// let mut map: CMap2<f64> = CMapBuilder::default()
    ///                             .n_darts(2)
    ///                             .build()
    ///                             .unwrap();
    /// map.two_link(1, 2);
    /// map.insert_vertex(1, (0.0, 0.0));
    /// map.insert_vertex(2, (1.0, 0.0));
    /// // split
    /// let nds = map.add_free_darts(6);
    /// let new_darts: Vec<_> = (nds..nds+6).collect();
    /// map.splitn_edge(1, &new_darts ,&[0.25, 0.50, 0.75]);
    /// // after
    /// //    <-<-<-<
    /// //  1 -3-4-5- 2
    /// //    >->->->
    /// assert_eq!(&new_darts[0..3], &[3, 4, 5]);
    /// assert_eq!(map.vertex(3), Some(Vertex2(0.25, 0.0)));
    /// assert_eq!(map.vertex(4), Some(Vertex2(0.50, 0.0)));
    /// assert_eq!(map.vertex(5), Some(Vertex2(0.75, 0.0)));
    ///
    /// assert_eq!(map.beta::<1>(1), 3);
    /// assert_eq!(map.beta::<1>(3), 4);
    /// assert_eq!(map.beta::<1>(4), 5);
    /// assert_eq!(map.beta::<1>(5), NULL_DART_ID);
    ///
    /// assert_eq!(map.beta::<1>(2), 6);
    /// assert_eq!(map.beta::<1>(6), 7);
    /// assert_eq!(map.beta::<1>(7), 8);
    /// assert_eq!(map.beta::<1>(8), NULL_DART_ID);
    ///
    /// assert_eq!(map.beta::<2>(1), 8);
    /// assert_eq!(map.beta::<2>(3), 7);
    /// assert_eq!(map.beta::<2>(4), 6);
    /// assert_eq!(map.beta::<2>(5), 2);
    /// ```
    pub fn splitn_edge(
        &mut self,
        edge_id: EdgeIdentifier,
        new_darts: &[DartIdentifier],
        midpoint_vertices: &[T],
    ) {
        // check pre-allocated darts reqs
        let n_t = midpoint_vertices.len();
        let n_d = new_darts.len();
        if n_d != 2 * (n_t + 1) {
            println!("W: inconsistent number of darts ({n_d}) & number of midpoints ({n_t}) - the method expects `2 * (n_mid + 1)` darts");
            println!("   skipping split...");
            return;
        }
        if new_darts.iter().any(|d| !self.is_free(*d)) {
            println!("W: all pre-allocated darts should be free");
            println!("   skipping split...");
            return;
        }

        // base darts making up the edge
        let base_dart1 = edge_id as DartIdentifier;
        let base_dart2 = self.beta::<2>(base_dart1);
        let b1d1_old = self.beta::<1>(base_dart1);

        // (*): unwrapping is ok since splitting an edge that does not have both its vertices
        //      defined is undefined behavior, therefore panic
        let v1 = self // (*)
            .vertex(self.vertex_id(base_dart1))
            .expect("E: attempt to split an edge that is not fully defined in the first place");
        let v2 = self // (*)
            .vertex(self.vertex_id(if base_dart2 == NULL_DART_ID {
                b1d1_old
            } else {
                base_dart2
            }))
            .expect("E: attempt to split an edge that is not fully defined in the first place");
        let seg = v2 - v1;

        // unsew current dart
        // self.one_unlink(base_dart1);
        self.betas[base_dart1 as usize][1] = 0;
        self.betas[b1d1_old as usize][0] = 0;
        if base_dart2 != NULL_DART_ID {
            self.two_unlink(base_dart1);
        }
        // insert new vertices / darts on base_dart1's side
        let mut prev_d = base_dart1;
        let darts = &new_darts[0..=n_t];
        if darts.iter().any(|d| *d == NULL_DART_ID) {
            println!("W: the null dart cannot be used to split an existing edge");
            println!("   skipping split...");
            return;
        }
        midpoint_vertices
            .iter()
            .zip(darts.iter())
            .for_each(|(&t, &new_d)| {
                if (t >= T::one()) | (t <= T::zero()) {
                    println!(
                        "W: vertex placement for split is not in ]0;1[ -- result may be incoherent"
                    );
                }
                let new_v = v1 + seg * t;
                self.one_link(prev_d, new_d);
                self.insert_vertex(new_d, new_v);
                prev_d = new_d;
            });
        self.one_link(prev_d, b1d1_old);

        // if b2(base_dart1) is defined, insert vertices / darts on its side too
        if base_dart2 != NULL_DART_ID {
            let other_darts = &new_darts[n_t + 1..];
            if other_darts.iter().any(|d| *d == NULL_DART_ID) {
                println!("W: the null dart cannot be used to split an existing edge");
                println!("   skipping split...");
                return;
            }
            let b1d2_old = self.beta::<1>(base_dart2);
            // self.one_unlink(base_dart2);
            self.betas[base_dart2 as usize][1] = 0;
            self.betas[b1d2_old as usize][0] = 0;
            let mut prev_d = base_dart2;
            darts
                .iter()
                .rev()
                .zip(other_darts.iter())
                .for_each(|(d, new_d)| {
                    self.two_link(prev_d, *d);
                    self.one_link(prev_d, *new_d);
                    prev_d = *new_d;
                });
            self.one_link(prev_d, b1d2_old);
            self.two_link(prev_d, base_dart1);
        }
    }
}
