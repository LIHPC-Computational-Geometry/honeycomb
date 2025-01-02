//! standard and no-alloc variants of the `splitn_edge` functions

// ------ IMPORTS

use crate::splits::SplitEdgeError;
use honeycomb_core::cmap::{CMap2, DartIdType, EdgeIdType, NULL_DART_ID};
use honeycomb_core::geometry::CoordsFloat;
// ------ CONTENT

#[allow(clippy::missing_errors_doc)]
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
/// # Return / Errors
///
/// This method will return:
/// - `Ok(())` if the operation is successful & the edge was split
/// - `Err(SplitEdgeError)` if the operation fails & the edge is left unchanged. Causes of failure
///   are described in [`SplitEdgeError`]'s documentation.
///
/// # Example
///
/// ```
/// # use honeycomb_core::prelude::{CMap2, CMapBuilder, NULL_DART_ID, Vertex2};
/// # use honeycomb_kernels::splits::splitn_edge;
/// // before
/// //    <--2---
/// //  1         2
/// //    ---1-->
/// let mut map: CMap2<f64> = CMapBuilder::default()
///                             .n_darts(2)
///                             .build()
///                             .unwrap();
/// map.force_link::<2>(1, 2);
/// map.force_write_vertex(1, (0.0, 0.0));
/// map.force_write_vertex(2, (1.0, 0.0));
/// // split
/// assert!(splitn_edge(&mut map, 1, [0.25, 0.50, 0.75]).is_ok());
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
/// assert_eq!(map.force_read_vertex(3), Some(Vertex2(0.25, 0.0)));
/// assert_eq!(map.force_read_vertex(4), Some(Vertex2(0.50, 0.0)));
/// assert_eq!(map.force_read_vertex(5), Some(Vertex2(0.75, 0.0)));
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
#[allow(clippy::cast_possible_truncation)]
pub fn splitn_edge<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    edge_id: EdgeIdType,
    midpoint_vertices: impl IntoIterator<Item = T>,
) -> Result<(), SplitEdgeError> {
    // check pre-allocated darts reqs
    let midpoint_vertices = midpoint_vertices.into_iter().collect::<Vec<_>>();
    let n_t = midpoint_vertices.len();

    // base darts making up the edge
    let base_dart1 = edge_id as DartIdType;
    let base_dart2 = cmap.beta::<2>(base_dart1);

    let new_darts = if base_dart2 == NULL_DART_ID {
        let tmp = cmap.add_free_darts(n_t);
        (tmp..tmp + n_t as DartIdType)
            .chain((0..n_t).map(|_| NULL_DART_ID))
            .collect::<Vec<_>>()
    } else {
        let tmp = cmap.add_free_darts(2 * n_t);
        (tmp..tmp + 2 * n_t as DartIdType).collect::<Vec<_>>()
    };
    // get the first and second halves
    let (darts_fh, darts_sh) = (&new_darts[..n_t], &new_darts[n_t..]);

    inner_splitn(cmap, base_dart1, darts_fh, darts_sh, &midpoint_vertices)
}

#[allow(clippy::missing_errors_doc)]
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
/// # Return / Errors
///
/// This method will return:
/// - `Ok(())` if the operation is successful & the edge was split
/// - `Err(SplitEdgeError)` if the operation fails & the edge is left unchanged. Causes of failure
///   are described in [`SplitEdgeError`]'s documentation and in requirements mentionned above.
pub fn splitn_edge_no_alloc<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    edge_id: EdgeIdType,
    new_darts: &[DartIdType],
    midpoint_vertices: &[T],
) -> Result<(), SplitEdgeError> {
    // check pre-allocated darts reqs
    let n_t = midpoint_vertices.len();
    let n_d = new_darts.len();
    if n_d != 2 * n_t {
        return Err(SplitEdgeError::WrongAmountDarts(2 * n_t, n_d));
    }
    if new_darts.iter().any(|d| !cmap.is_free(*d)) {
        return Err(SplitEdgeError::InvalidDarts("one dart is not free"));
    }
    // get the first and second halves
    let darts_fh = &new_darts[..n_t];
    let darts_sh = &new_darts[n_t..];

    // base darts making up the edge
    let base_dart1 = edge_id as DartIdType;
    let base_dart2 = cmap.beta::<2>(base_dart1);

    if darts_fh.iter().any(|d| *d == NULL_DART_ID) {
        return Err(SplitEdgeError::InvalidDarts(
            "one dart of the first half is null",
        ));
    }
    if base_dart2 != NULL_DART_ID && darts_sh.iter().any(|d| *d == NULL_DART_ID) {
        return Err(SplitEdgeError::InvalidDarts(
            "one dart of the second half is null",
        ));
    }

    inner_splitn(cmap, base_dart1, darts_fh, darts_sh, midpoint_vertices)
}

// --- common inner routine

fn inner_splitn<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    base_dart1: DartIdType,
    darts_fh: &[DartIdType], //first half
    darts_sh: &[DartIdType], //second half
    midpoint_vertices: &[T],
) -> Result<(), SplitEdgeError> {
    if midpoint_vertices
        .iter()
        .any(|t| (*t >= T::one()) | (*t <= T::zero()))
    {
        return Err(SplitEdgeError::VertexBound);
    }

    let base_dart2 = cmap.beta::<2>(base_dart1);
    let b1d1_old = cmap.beta::<1>(base_dart1);

    let (Some(v1), Some(v2)) = (
        cmap.force_read_vertex(cmap.vertex_id(base_dart1)),
        cmap.force_read_vertex(cmap.vertex_id(if base_dart2 == NULL_DART_ID {
            b1d1_old
        } else {
            base_dart2
        })),
    ) else {
        return Err(SplitEdgeError::UndefinedEdge);
    };
    let seg = v2 - v1;

    // unsew current dart
    // self.one_unlink(base_dart1);
    cmap.set_beta::<1>(base_dart1, 0);
    cmap.set_beta::<0>(b1d1_old, 0);
    if base_dart2 != NULL_DART_ID {
        cmap.force_unlink::<2>(base_dart1);
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
            cmap.force_link::<1>(prev_d, new_d);
            cmap.force_write_vertex(new_d, new_v);
            prev_d = new_d;
        });
    cmap.force_link::<1>(prev_d, b1d1_old);

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
                cmap.force_link::<2>(prev_d, *d);
                cmap.force_link::<1>(prev_d, *new_d);
                prev_d = *new_d;
            });
        cmap.force_link::<1>(prev_d, b1d2_old);
        cmap.force_link::<2>(prev_d, base_dart1);
    }

    Ok(())
}
