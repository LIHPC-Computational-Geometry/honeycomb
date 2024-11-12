//! Step 2 implementation
//!
//! The main goal of this step is tp precompute information to:
//! - parallelize step 3
//! - make step 3 and step 4 independent from each other

// ------ IMPORTS

use super::{DartSlices, IntersectionsPerEdge};
use honeycomb_core::prelude::{CMap2, CoordsFloat, DartIdType, EdgeIdType, NULL_DART_ID};
use std::collections::HashMap;

// ------ CONTENT

pub(crate) fn group_intersections_per_edge<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    intersection_metadata: Vec<(DartIdType, T)>,
) -> (IntersectionsPerEdge<T>, DartSlices) {
    // group intersection data per edge, and associate an ID to each
    let mut edge_intersec: HashMap<EdgeIdType, Vec<(usize, T, DartIdType)>> = HashMap::new();
    intersection_metadata
        .into_iter()
        .filter(|(_, t)| !t.is_nan())
        .enumerate()
        .for_each(|(idx, (dart_id, mut t))| {
            // classify intersections per edge_id & adjust t if  needed
            let edge_id = cmap.edge_id(dart_id);
            // condition works in 2D because edges are 2 darts at most
            if edge_id != dart_id {
                t = T::one() - t;
            }
            if let Some(storage) = edge_intersec.get_mut(&edge_id) {
                // not the first intersction with this given edge
                storage.push((idx, t, dart_id));
            } else {
                // first intersction with this given edge
                edge_intersec.insert(edge_id, vec![(idx, t, dart_id)]);
            }
        });

    // sort per t for later
    for vs in edge_intersec.values_mut() {
        // panic unreachable because t s.t. t == NaN have been filtered previously
        vs.sort_by(|(_, t1, _), (_, t2, _)| t1.partial_cmp(t2).expect("E: unreachable"));
    }

    // prealloc darts that will be used for vertex insertion
    let n_darts_per_seg: Vec<_> = edge_intersec.values().map(|vs| 2 * vs.len()).collect();
    let n_tot: usize = n_darts_per_seg.iter().sum();
    let tmp = cmap.add_free_darts(n_tot) as usize;
    // the prefix sum gives an offset that corresponds to the starting index of each slice, minus
    // the location of the allocated dart block (given by `tmp`)
    // end of the slice is deduced using these values and the number of darts the current seg needs
    let prefix_sum: Vec<usize> = n_darts_per_seg
        .iter()
        .scan(0, |state, &n_d| {
            *state += n_d;
            Some(*state - n_d) // we want an offset, not the actual sum
        })
        .collect();

    #[allow(clippy::cast_possible_truncation)]
    let dart_slices: Vec<Vec<DartIdType>> = n_darts_per_seg
        .iter()
        .zip(prefix_sum.iter())
        .map(|(n_d, start)| {
            ((tmp + start) as DartIdType..(tmp + start + n_d) as DartIdType).collect::<Vec<_>>()
        })
        .collect();

    (edge_intersec, dart_slices)
}

pub(crate) fn compute_intersection_ids<T: CoordsFloat>(
    n_intersec: usize,
    edge_intersec: &IntersectionsPerEdge<T>,
    dart_slices: &DartSlices,
) -> Vec<DartIdType> {
    let mut res = vec![NULL_DART_ID; n_intersec];
    for ((edge_id, vs), new_darts) in edge_intersec.iter().zip(dart_slices.iter()) {
        // order should be consistent between collection because of the sort_by call
        let hl = new_darts.len() / 2; // half-length; also equal to n_intermediate
        let fh = &new_darts[..hl]; // first half;  used for the side of edge id
        let sh = &new_darts[hl..]; // second half; used for the opposite side
        for (i, (id, _, old_dart_id)) in vs.iter().enumerate() {
            // readjust according to intersection side
            res[*id] = if *old_dart_id == *edge_id {
                fh[i]
            } else {
                sh[hl - 1 - i]
            };
        }
    }
    res
}
