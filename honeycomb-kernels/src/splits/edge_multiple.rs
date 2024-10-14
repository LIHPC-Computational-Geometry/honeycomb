//! standard and no-alloc variants of the `splitn_edge` functions

// ------ IMPORTS

use honeycomb_core::cmap::{CMap2, DartIdentifier, EdgeIdentifier, NULL_DART_ID};
use honeycomb_core::geometry::CoordsFloat;

// ------ CONTENT

/// Split an edge into `n` segments.
///
/// <div class="warning">
/// This implementation is 2D specific.
/// </div>
///
/// # Arguments
///
/// - `cmap: &mut CMap2<T>` -- Reference to the modified map.
/// - `edge_id: EdgeIdentifier` -- Edge to split in two.
/// - `midpoint_vertices: I` -- Relative positions of new vertices, starting from the
///   vertex of the dart sharing `edge_id` as its identifier.
///
/// ## Generics
///
/// - `I: Iterator<Item = T>` -- Iterator over `T` values. These should be in the `]0; 1[` open range.
///
/// # Return
///
/// This method will return:
/// - `true` if the operation is successful & the edge was split
/// - `false` if the operation fails & the edge is left unchanged. It can fail if:
///   - one or both vertices of the edge is undefined
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
/// assert!(map.splitn_edge(1, [0.25, 0.50, 0.75]));
/// // after
/// //    <-<-<-<
/// //  1 -3-4-5- 2
/// //    >->->->
/// let new_darts = [
///     map.beta::<1>(1),
///     map.beta::<1>(map.beta::<1>(1)),
///     map.beta::<1>(map.beta::<1>(map.beta::<1>(1))),
/// ];
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
pub fn splitn_edge<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    edge_id: EdgeIdentifier,
    midpoint_vertices: impl IntoIterator<Item = T>,
) -> bool {
    // check pre-allocated darts reqs
    let midpoint_vertices = midpoint_vertices.into_iter().collect::<Vec<_>>();
    let n_t = midpoint_vertices.len();

    // base darts making up the edge
    let base_dart1 = edge_id as DartIdentifier;
    let base_dart2 = cmap.beta::<2>(base_dart1);

    let new_darts = if base_dart2 == NULL_DART_ID {
        let tmp = cmap.add_free_darts(n_t);
        (tmp..tmp + n_t as DartIdentifier)
            .chain((0..n_t).map(|_| NULL_DART_ID))
            .collect::<Vec<_>>()
    } else {
        let tmp = cmap.add_free_darts(2 * n_t);
        (tmp..tmp + 2 * n_t as DartIdentifier).collect::<Vec<_>>()
    };
    // get the first and second halves
    let (darts_fh, darts_sh) = (&new_darts[..n_t], &new_darts[n_t..]);

    inner_splitn(cmap, base_dart1, darts_fh, darts_sh, &midpoint_vertices)
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
/// - `cmap: &mut CMap2<T>` -- Reference to the modified map.
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
/// # Return
///
/// This method will return:
/// - `true` if the operation is successful & the edge was split
/// - `false` if the operation fails & the edge is left unchanged. It can fail if:
///   - one or both vertices of the edge is undefined
///   - if darts passed as argument do not match the above requirements
pub fn splitn_edge_no_alloc<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    edge_id: EdgeIdentifier,
    new_darts: &[DartIdentifier],
    midpoint_vertices: &[T],
) -> bool {
    // check pre-allocated darts reqs
    let n_t = midpoint_vertices.len();
    let n_d = new_darts.len();
    if n_d != 2 * n_t {
        // println!("W: inconsistent number of darts ({n_d}) & number of midpoints ({n_t}) - the method expects `2 * n_mid` darts");
        // println!("{SKIP}");
        return false;
    }
    if new_darts.iter().any(|d| !cmap.is_free(*d)) {
        // println!("{W_PASSED_NONFREE}");
        // println!("{SKIP}");
        return false;
    }
    // get the first and second halves
    let darts_fh = &new_darts[..n_t];
    let darts_sh = &new_darts[n_t..];

    // base darts making up the edge
    let base_dart1 = edge_id as DartIdentifier;
    let base_dart2 = cmap.beta::<2>(base_dart1);

    if darts_fh.iter().any(|d| *d == NULL_DART_ID) {
        // println!("{W_PASSED_NULL}");
        // println!("{SKIP}");
        return false;
    }
    if base_dart2 != NULL_DART_ID && darts_sh.iter().any(|d| *d == NULL_DART_ID) {
        // println!("{W_PASSED_NULL}");
        // println!("{SKIP}");
        return false;
    }

    inner_splitn(cmap, base_dart1, darts_fh, darts_sh, midpoint_vertices)
}

// --- common inner routine

fn inner_splitn<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    base_dart1: DartIdentifier,
    darts_fh: &[DartIdentifier], //first half
    darts_sh: &[DartIdentifier], //second half
    midpoint_vertices: &[T],
) -> bool {
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
        // println!("{W_UNDEF_EDGE}");
        // println!("{SKIP}");
        return false;
    };
    let seg = v2 - v1;

    // unsew current dart
    // self.one_unlink(base_dart1);
    cmap.set_beta::<1>(base_dart1, 0);
    cmap.set_beta::<0>(b1d1_old, 0);
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
                // println!("{W_VERTEX_BOUND}");
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
        cmap.set_beta::<1>(base_dart2, 0);
        cmap.set_beta::<0>(b1d2_old, 0);
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

    true
}
