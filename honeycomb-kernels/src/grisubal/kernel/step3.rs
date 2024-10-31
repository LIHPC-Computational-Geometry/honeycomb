//! Step 3 implementation
//!

// ------ IMPORTS

use super::{DartSlices, IntersectionsPerEdge};
use crate::splits::splitn_edge_no_alloc;
use honeycomb_core::prelude::{CMap2, CoordsFloat};

// ------ CONTENT

pub(crate) fn insert_intersections<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    edge_intersec: &IntersectionsPerEdge<T>,
    dart_slices: &DartSlices,
) {
    for ((edge_id, vs), new_darts) in edge_intersec.iter().zip(dart_slices.iter()) {
        let _ = splitn_edge_no_alloc(
            cmap,
            *edge_id,
            new_darts,
            &vs.iter().map(|(_, t, _)| *t).collect::<Vec<_>>(),
        );
    }
}
