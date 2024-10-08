//! Advanced operations implementation
//!
//! This module contains code used to implement advanced operations, e.g. some non-standard,
//! higher-level abstractions that are useful in meshing algorithms.

// ------ IMPORTS

use crate::geometry::CoordsFloat;
use crate::prelude::{CMap2, DartIdentifier, EdgeIdentifier, Vertex2, NULL_DART_ID};

// ------ CONTENT

/// **Advanced operations: edge splitting -- standard variants**
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
    pub fn split_edge(&mut self, edge_id: EdgeIdentifier, midpoint_vertex: Option<T>) {
        // midpoint check
        if midpoint_vertex.is_some_and(|t| (t >= T::one()) | (t <= T::zero())) {
            println!("W: vertex placement for split is not in ]0;1[ -- result may be incoherent");
        }

        // base darts making up the edge
        let base_dart1 = edge_id as DartIdentifier;
        let base_dart2 = self.beta::<2>(base_dart1);

        let new_darts = if base_dart2 == NULL_DART_ID {
            (self.add_free_dart(), NULL_DART_ID)
        } else {
            let tmp = self.add_free_darts(2);
            (tmp, tmp + 1)
        };

        inner_split(self, base_dart1, new_darts, midpoint_vertex);
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
    /// let new_darts = map.splitn_edge(1, [0.25, 0.50, 0.75]);
    /// // after
    /// //    <-<-<-<
    /// //  1 -3-4-5- 2
    /// //    >->->->
    /// assert_eq!(&new_darts, &[3, 4, 5]);
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
        midpoint_vertices: impl IntoIterator<Item = T>,
    ) -> Vec<DartIdentifier> {
        // check pre-allocated darts reqs
        let midpoint_vertices = midpoint_vertices.into_iter().collect::<Vec<_>>();
        let n_t = midpoint_vertices.len();

        // base darts making up the edge
        let base_dart1 = edge_id as DartIdentifier;
        let base_dart2 = self.beta::<2>(base_dart1);

        let new_darts = if base_dart2 == NULL_DART_ID {
            let tmp = self.add_free_darts(n_t);
            (tmp..tmp + n_t as DartIdentifier)
                .chain((0..n_t).map(|_| NULL_DART_ID))
                .collect::<Vec<_>>()
        } else {
            let tmp = self.add_free_darts(2 * n_t);
            (tmp..tmp + 2 * n_t as DartIdentifier).collect::<Vec<_>>()
        };
        // get the first and second halves
        let (darts_fh, darts_sh) = (&new_darts[..n_t], &new_darts[n_t..]);

        inner_splitn(self, base_dart1, darts_fh, darts_sh, &midpoint_vertices);

        darts_fh.to_vec()
    }
}

/// **Advanced operations: edge splitting -- no allocation variants**
impl<T: CoordsFloat> CMap2<T> {
    /// Split an edge into two segments.
    ///
    /// <div class="warning">
    /// This implementation is 2D specific.
    /// </div>
    ///
    /// This method is a variant of [`split_edge`] where inline dart allocations are removed. The
    /// aim of this variant is to enhance performance by enabling the user to pre-allocate a number
    /// of darts.
    ///
    /// The method follows the same logic as the regular [`split_edge`], the only difference being
    /// that the new darts won't be added to the map on the fly. Instead, the method uses the two
    /// darts passed as argument (`new_darts`) to build the new segments. Consequently, there is no
    /// guarantee that IDs will be consistent between this and the regular method.
    ///
    /// # Arguments
    ///
    /// - `edge_id: EdgeIdentifier` -- Edge to split in two.
    /// - `new_darts: (DartIdentifier, DartIdentifier)` -- Dart IDs used to build the new segments.
    /// - `midpoint_vertex: Option<T>` -- Relative position of the new vertex, starting from the
    ///   vertex of the dart sharing `edge_id` as its identifier.
    ///
    /// ## Dart IDs Requirements & Usage
    ///
    /// Because of the dimension, the number of dart needed to perform this operation is at most
    /// two. These are the requirements for these two darts:
    /// - identifiers are passed as a tuple.
    /// - the first dart of the tuple will always be used if the operation is successful.
    /// - the second dart of the tuple will only be used if the original edge is made of two darts;
    ///   if that is not the case, the second dart ID can be `NULL_DART_ID`.
    /// - both of these darts should be free
    ///
    /// # Panics
    ///
    /// This method may panic if the edge upon which the operation is performed does not have two
    /// defined vertices.
    pub fn split_edge_noalloc(
        &mut self,
        edge_id: EdgeIdentifier,
        new_darts: (DartIdentifier, DartIdentifier), // 2D => statically known number of darts
        midpoint_vertex: Option<T>,
    ) {
        // midpoint check
        if midpoint_vertex.is_some_and(|t| (t >= T::one()) | (t <= T::zero())) {
            println!("W: vertex placement for split is not in ]0;1[ -- result may be incoherent");
        }

        // base darts making up the edge
        let base_dart1 = edge_id as DartIdentifier;
        let base_dart2 = self.beta::<2>(base_dart1);

        let (b1d1_new, b1d2_new) = new_darts;
        if new_darts.0 == NULL_DART_ID || !self.is_free(new_darts.0) {
            println!("W: dart {b1d1_new} cannot be used in split_edge -- passed darts should be non-null and free");
            println!("   skipping split...");
            return;
        }
        if base_dart2 != NULL_DART_ID && (b1d2_new == NULL_DART_ID || !self.is_free(b1d2_new)) {
            println!("W: dart {b1d2_new} cannot be used in split_edge -- passed darts should be non-null and free");
            println!("   skipping split...");
            return;
        }

        inner_split(self, base_dart1, new_darts, midpoint_vertex);
    }

    /// Split an edge into `n` segments.
    ///
    /// <div class="warning">
    /// This implementation is 2D specific.
    /// </div>
    ///
    /// This method is a variant of [`splitn_edge`] where inline dart allocations are removed. The
    /// aim of this variant is to enhance performance by enabling the user to pre-allocate a number
    /// of darts.
    ///
    /// The method follows the same logic as the regular [`splitn_edge`], the only difference being
    /// that the new darts won't be added to the map on the fly. Instead, the method uses darts
    /// passed as argument (`new_darts`) to build the new segments. Consequently, there is no
    /// guarantee that IDs will be consistent between this and the regular method.
    ///
    /// # Arguments
    ///
    /// - `edge_id: EdgeIdentifier` -- Edge to split in two.
    /// - `new_darts: &[DartIdentifier]` -- Dart IDs used to build the new segments.
    /// - `midpoint_vertices: &[T]` -- Relative positions of new vertices, starting from the
    ///   vertex of the dart sharing `edge_id` as its identifier.
    ///
    /// ## Dart IDs Requirements & Usage
    ///
    /// Because of the dimension, we can easily compute the number of dart needed to perform this
    /// operation. These are the requirements for the darts:
    /// - identifiers are passed as a slice:
    ///   - slice length should verify `new_darts.len() == 2 * midpoint_vertices.len()`
    /// - the first half of the slice will always be used if the operation is successful.
    /// - the second half of the slice will only be used if the original edge is made of two darts;
    ///   if that is not the case, the second half IDs can all be `NULL_DART_ID`s.
    /// - all of these darts should be free
    ///
    /// # Panics
    ///
    /// This method may panic if the edge upon which the operation is performed does not have two defined vertices.
    pub fn splitn_edge_no_alloc(
        &mut self,
        edge_id: EdgeIdentifier,
        new_darts: &[DartIdentifier],
        midpoint_vertices: &[T],
    ) {
        // check pre-allocated darts reqs
        let n_t = midpoint_vertices.len();
        let n_d = new_darts.len();
        if n_d != 2 * n_t {
            println!("W: inconsistent number of darts ({n_d}) & number of midpoints ({n_t}) - the method expects `2 * n_mid` darts");
            println!("   skipping split...");
            return;
        }
        if new_darts.iter().any(|d| !self.is_free(*d)) {
            println!("W: all pre-allocated darts should be free");
            println!("   skipping split...");
            return;
        }
        // get the first and second halves
        let darts_fh = &new_darts[..n_t];
        let darts_sh = &new_darts[n_t..];

        // base darts making up the edge
        let base_dart1 = edge_id as DartIdentifier;
        let base_dart2 = self.beta::<2>(base_dart1);

        if darts_fh.iter().any(|d| *d == NULL_DART_ID) {
            println!("W: the null dart cannot be used to split an existing edge");
            println!("   skipping split...");
            return;
        }
        if base_dart2 != NULL_DART_ID && darts_sh.iter().any(|d| *d == NULL_DART_ID) {
            println!("W: the null dart cannot be used to split an existing edge");
            println!("   skipping split...");
            return;
        }

        inner_splitn(self, base_dart1, darts_fh, darts_sh, midpoint_vertices);
    }
}

// --- common inner routine

fn inner_split<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    base_dart1: DartIdentifier,
    new_darts: (DartIdentifier, DartIdentifier), // 2D => statically known number of darts
    midpoint_vertex: Option<T>,
) {
    // base darts making up the edge
    let base_dart2 = cmap.beta::<2>(base_dart1);
    // (*): unwrapping is ok since splitting an edge that does not have both its vertices
    //      defined is undefined behavior, therefore panic
    if base_dart2 == NULL_DART_ID {
        let b1d1_old = cmap.beta::<1>(base_dart1);
        let b1d1_new = new_darts.0;
        let (Some(v1), Some(v2)) = (
            cmap.vertex(cmap.vertex_id(base_dart1)),
            cmap.vertex(cmap.vertex_id(b1d1_old)),
        ) else {
            println!("W: attempt to split an edge that is not fully defined in the first place");
            println!("   skipping split...");
            return;
        };
        /*
        let v1 = cmap
            .vertex(cmap.vertex_id(base_dart1))
            .expect("E: attempt to split an edge that is not fully defined in the first place");
        let v2 = cmap // (*)
            .vertex(cmap.vertex_id(b1d1_old))
            .expect("E: attempt to split an edge that is not fully defined in the first place");
        */
        // unsew current dart
        // self.one_unlink(base_dart1);
        cmap.betas[base_dart1 as usize][1] = 0;
        cmap.betas[b1d1_old as usize][0] = 0;
        // rebuild the edge
        cmap.one_link(base_dart1, b1d1_new);
        cmap.one_link(b1d1_new, b1d1_old);
        // insert the new vertex
        let seg = v2 - v1;
        cmap.insert_vertex(
            cmap.vertex_id(b1d1_new),
            midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
        );
    } else {
        let b1d1_old = cmap.beta::<1>(base_dart1);
        let b1d2_old = cmap.beta::<1>(base_dart2);
        let (b1d1_new, b1d2_new) = new_darts;
        let (Some(v1), Some(v2)) = (
            cmap.vertex(cmap.vertex_id(base_dart1)),
            cmap.vertex(cmap.vertex_id(base_dart2)),
        ) else {
            println!("W: attempt to split an edge that is not fully defined in the first place");
            println!("   skipping split...");
            return;
        };
        /*
        let v1 = cmap // (*)
            .vertex(cmap.vertex_id(base_dart1))
            .expect("E: attempt to split an edge that is not fully defined in the first place");
        let v2 = cmap // (*)
            .vertex(cmap.vertex_id(base_dart2))
            .expect("E: attempt to split an edge that is not fully defined in the first place");
        */
        // unsew current darts
        // self.one_unlink(base_dart1);
        cmap.betas[base_dart1 as usize][1] = 0;
        cmap.betas[b1d1_old as usize][0] = 0;
        // self.one_unlink(base_dart2);
        cmap.betas[base_dart2 as usize][1] = 0;
        cmap.betas[b1d2_old as usize][0] = 0;
        cmap.two_unlink(base_dart1);
        // rebuild the edge
        cmap.one_link(base_dart1, b1d1_new);
        if b1d1_old != NULL_DART_ID {
            cmap.one_link(b1d1_new, b1d1_old);
        }
        cmap.one_link(base_dart2, b1d2_new);
        if b1d2_old != NULL_DART_ID {
            cmap.one_link(b1d2_new, b1d2_old);
        }
        cmap.two_link(base_dart1, b1d2_new);
        cmap.two_link(base_dart2, b1d1_new);
        // insert the new vertex
        let seg = v2 - v1;
        cmap.insert_vertex(
            cmap.vertex_id(b1d1_new),
            midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
        );
    }
}

fn inner_splitn<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    base_dart1: DartIdentifier,
    darts_fh: &[DartIdentifier], //first half
    darts_sh: &[DartIdentifier], //second half
    midpoint_vertices: &[T],
) {
    let base_dart2 = cmap.beta::<2>(base_dart1);
    let b1d1_old = cmap.beta::<1>(base_dart1);

    // (*): unwrapping is ok since splitting an edge that does not have both its vertices
    //      defined is undefined behavior, therefore panic
    let (Some(v1), Some(v2)) = (
        cmap.vertex(cmap.vertex_id(base_dart1)),
        cmap.vertex(cmap.vertex_id(if base_dart2 == NULL_DART_ID {
            b1d1_old
        } else {
            base_dart2
        })),
    ) else {
        println!("W: attempt to split an edge that is not fully defined in the first place");
        println!("   skipping split...");
        return;
    };
    /*
    let v1 = cmap // (*)
        .vertex(cmap.vertex_id(base_dart1))
        .expect("E: attempt to split an edge that is not fully defined in the first place");
    let v2 = cmap // (*)
        .vertex(cmap.vertex_id(if base_dart2 == NULL_DART_ID {
            b1d1_old
        } else {
            base_dart2
        }))
        .expect("E: attempt to split an edge that is not fully defined in the first place");
    */
    let seg = v2 - v1;

    // unsew current dart
    // self.one_unlink(base_dart1);
    cmap.betas[base_dart1 as usize][1] = 0;
    cmap.betas[b1d1_old as usize][0] = 0;
    if base_dart2 != NULL_DART_ID {
        cmap.two_unlink(base_dart1);
    }
    // insert new vertices / darts on base_dart1's side
    let mut prev_d = base_dart1;
    midpoint_vertices
        .iter()
        .zip(darts_fh.iter())
        .for_each(|(&t, &new_d)| {
            if (t >= T::one()) | (t <= T::zero()) {
                println!(
                    "W: vertex placement for split is not in ]0;1[ -- result may be incoherent"
                );
            }
            let new_v = v1 + seg * t;
            cmap.one_link(prev_d, new_d);
            cmap.insert_vertex(new_d, new_v);
            prev_d = new_d;
        });
    cmap.one_link(prev_d, b1d1_old);

    // if b2(base_dart1) is defined, insert vertices / darts on its side too
    if base_dart2 != NULL_DART_ID {
        let b1d2_old = cmap.beta::<1>(base_dart2);
        // self.one_unlink(base_dart2);
        cmap.betas[base_dart2 as usize][1] = 0;
        cmap.betas[b1d2_old as usize][0] = 0;
        let mut prev_d = base_dart2;
        darts_fh
            .iter()
            .rev()
            .zip(darts_sh.iter())
            .for_each(|(d, new_d)| {
                cmap.two_link(prev_d, *d);
                cmap.one_link(prev_d, *new_d);
                prev_d = *new_d;
            });
        cmap.one_link(prev_d, b1d2_old);
        cmap.two_link(prev_d, base_dart1);
    }
}
