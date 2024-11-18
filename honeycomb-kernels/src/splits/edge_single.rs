//! standard and no-alloc variants of the `split_edge` functions

// ------ IMPORTS

use crate::splits::SplitEdgeError;
use honeycomb_core::cmap::{CMap2, DartIdType, EdgeIdType, NULL_DART_ID};
use honeycomb_core::geometry::{CoordsFloat, Vertex2};

// ------ CONTENT

#[allow(clippy::missing_errors_doc)]
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
/// - `cmap: &mut CMap2<T>` -- Reference to the modified map.
/// - `edge_id: EdgeIdentifier` -- Edge to split in two.
/// - `midpoint_vertex: Option<T>` -- Relative position of the new vertex, starting from the
///   vertex of the dart sharing `edge_id` as its identifier.
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
pub fn split_edge<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    edge_id: EdgeIdType,
    midpoint_vertex: Option<T>,
) -> Result<(), SplitEdgeError> {
    // midpoint check
    if midpoint_vertex.is_some_and(|t| (t >= T::one()) | (t <= T::zero())) {
        return Err(SplitEdgeError::VertexBound);
    }

    // base darts making up the edge
    let base_dart1 = edge_id as DartIdType;
    let base_dart2 = cmap.beta::<2>(base_dart1);

    let new_darts = if base_dart2 == NULL_DART_ID {
        (cmap.add_free_dart(), NULL_DART_ID)
    } else {
        let tmp = cmap.add_free_darts(2);
        (tmp, tmp + 1)
    };

    inner_split(cmap, base_dart1, new_darts, midpoint_vertex)
}

#[allow(clippy::missing_errors_doc)]
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
/// - `cmap: &mut CMap2<T>` -- Reference to the modified map.
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
/// # Return / Errors
///
/// This method will return:
/// - `Ok(())` if the operation is successful & the edge was split
/// - `Err(SplitEdgeError)` if the operation fails & the edge is left unchanged. Causes of failure
///   are described in [`SplitEdgeError`]'s documentation and in requirements mentionned above.
pub fn split_edge_noalloc<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    edge_id: EdgeIdType,
    new_darts: (DartIdType, DartIdType), // 2D => statically known number of darts
    midpoint_vertex: Option<T>,
) -> Result<(), SplitEdgeError> {
    // midpoint check
    if midpoint_vertex.is_some_and(|t| (t >= T::one()) | (t <= T::zero())) {
        return Err(SplitEdgeError::VertexBound);
    }

    // base darts making up the edge
    let base_dart1 = edge_id as DartIdType;
    let base_dart2 = cmap.beta::<2>(base_dart1);

    if new_darts.0 == NULL_DART_ID || !cmap.is_free(new_darts.0) {
        return Err(SplitEdgeError::InvalidDarts(
            "first dart is null or not free",
        ));
    }
    if base_dart2 != NULL_DART_ID && (new_darts.1 == NULL_DART_ID || !cmap.is_free(new_darts.1)) {
        return Err(SplitEdgeError::InvalidDarts(
            "second dart is null or not free",
        ));
    }

    inner_split(cmap, base_dart1, new_darts, midpoint_vertex)
}

// --- common inner routine

fn inner_split<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    base_dart1: DartIdType,
    new_darts: (DartIdType, DartIdType), // 2D => statically known number of darts
    midpoint_vertex: Option<T>,
) -> Result<(), SplitEdgeError> {
    // base darts making up the edge
    let base_dart2 = cmap.beta::<2>(base_dart1);
    if base_dart2 == NULL_DART_ID {
        let b1d1_old = cmap.beta::<1>(base_dart1);
        let b1d1_new = new_darts.0;
        let (Some(v1), Some(v2)) = (
            cmap.vertex(cmap.vertex_id(base_dart1)),
            cmap.vertex(cmap.vertex_id(b1d1_old)),
        ) else {
            return Err(SplitEdgeError::UndefinedEdge);
        };
        // unsew current dart
        cmap.set_beta::<1>(base_dart1, 0);
        cmap.set_beta::<0>(b1d1_old, 0);
        // rebuild the edge
        cmap.force_one_link(base_dart1, b1d1_new);
        cmap.force_one_link(b1d1_new, b1d1_old);
        // insert the new vertex
        let seg = v2 - v1;
        cmap.insert_vertex(
            cmap.vertex_id(b1d1_new),
            midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
        );
        Ok(())
    } else {
        let b1d1_old = cmap.beta::<1>(base_dart1);
        let b1d2_old = cmap.beta::<1>(base_dart2);
        let (b1d1_new, b1d2_new) = new_darts;
        let (Some(v1), Some(v2)) = (
            cmap.vertex(cmap.vertex_id(base_dart1)),
            cmap.vertex(cmap.vertex_id(base_dart2)),
        ) else {
            return Err(SplitEdgeError::UndefinedEdge);
        };
        // unsew current darts
        cmap.set_beta::<1>(base_dart1, 0);
        cmap.set_beta::<0>(b1d1_old, 0);
        cmap.set_beta::<1>(base_dart2, 0);
        cmap.set_beta::<0>(b1d2_old, 0);
        cmap.force_two_unlink(base_dart1);
        // rebuild the edge
        cmap.force_one_link(base_dart1, b1d1_new);
        if b1d1_old != NULL_DART_ID {
            cmap.force_one_link(b1d1_new, b1d1_old);
        }
        cmap.force_one_link(base_dart2, b1d2_new);
        if b1d2_old != NULL_DART_ID {
            cmap.force_one_link(b1d2_new, b1d2_old);
        }
        cmap.force_two_link(base_dart1, b1d2_new);
        cmap.force_two_link(base_dart2, b1d1_new);
        // insert the new vertex
        let seg = v2 - v1;
        cmap.insert_vertex(
            cmap.vertex_id(b1d1_new),
            midpoint_vertex.map_or(Vertex2::average(&v1, &v2), |t| v1 + seg * t),
        );
        Ok(())
    }
}
