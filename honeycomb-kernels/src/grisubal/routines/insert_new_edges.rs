//! Step 5 implementation
//
//! Use the information computed at step 4 and insert all new edges into the map.

// ------ IMPORTS

use crate::grisubal::model::{Boundary, MapEdge};
use crate::splits::splitn_edge_no_alloc;
use honeycomb_core::prelude::{CMap2, CoordsFloat, DartIdType};

// ------ CONTENT

pub(crate) fn insert_edges_in_map<T: CoordsFloat>(cmap: &mut CMap2<T>, edges: &[MapEdge<T>]) {
    // FIXME: minimize allocs & redundant operations
    // prealloc all darts needed
    let n_darts_per_seg: Vec<_> = edges
        .iter()
        .map(|e| 2 + 2 * e.intermediates.len())
        .collect();
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

    // insert new edges
    for (
        MapEdge {
            start,
            intermediates,
            end,
        },
        dslice,
    ) in edges.iter().zip(dart_slices.iter())
    {
        // remove deprecated connectivities & save what data is necessary
        let b1_start_old = cmap.beta::<1>(*start);
        let b0_end_old = cmap.beta::<0>(*end);
        cmap.one_unlink(*start);
        cmap.one_unlink(b0_end_old);

        let &[d_new, b2_d_new] = &dslice[0..2] else {
            unreachable!()
        };
        cmap.two_link(d_new, b2_d_new);

        // rebuild - this is the final construct if there are no intermediates
        cmap.one_link(*start, d_new);
        cmap.one_link(b2_d_new, b1_start_old);
        cmap.one_link(d_new, *end);
        cmap.one_link(b0_end_old, b2_d_new);

        if !intermediates.is_empty() {
            // create the topology components
            let edge_id = cmap.edge_id(d_new);
            let new_darts = &dslice[2..];
            let _ = splitn_edge_no_alloc(
                cmap,
                edge_id,
                new_darts,
                &vec![T::from(0.5).unwrap(); intermediates.len()],
            );
            // replace placeholder vertices
            let mut dart_id = cmap.beta::<1>(edge_id as DartIdType);
            for v in intermediates {
                let vid = cmap.vertex_id(dart_id);
                let _ = cmap.replace_vertex(vid, *v);
                dart_id = cmap.beta::<1>(dart_id);
            }
        }

        let mut d_boundary = cmap.beta::<1>(*start);
        while d_boundary != *end {
            cmap.set_attribute::<Boundary>(d_boundary, Boundary::Left);
            cmap.set_attribute::<Boundary>(cmap.beta::<2>(d_boundary), Boundary::Right);
            d_boundary = cmap.beta::<1>(d_boundary);
        }
    }
}
