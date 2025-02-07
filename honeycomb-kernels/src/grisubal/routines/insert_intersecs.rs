//! Step 3 implementation
//!
//! Insert the intersections into the map.

// ------ IMPORTS

use super::{DartSlices, IntersectionsPerEdge};
use crate::splits::splitn_edge_transac;
use honeycomb_core::{
    prelude::{CMap2, CoordsFloat},
    stm::atomically_with_err,
};

// ------ CONTENT

pub(crate) fn insert_intersections<T: CoordsFloat>(
    cmap: &CMap2<T>,
    edge_intersec: &IntersectionsPerEdge<T>,
    dart_slices: &DartSlices,
) {
    for ((edge_id, vs), new_darts) in edge_intersec.iter().zip(dart_slices.iter()) {
        atomically_with_err(|trans| {
            splitn_edge_transac(
                cmap,
                trans,
                *edge_id,
                new_darts,
                &vs.iter().map(|(_, t, _)| *t).collect::<Vec<_>>(),
            )
        })
        .unwrap();
    }
}
