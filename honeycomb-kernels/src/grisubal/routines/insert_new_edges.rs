//! Step 5 implementation
//
//! Use the information computed at step 4 and insert all new edges into the map.

use honeycomb_core::cmap::{CMap2, DartIdType};
use honeycomb_core::geometry::CoordsFloat;
use honeycomb_core::stm::atomically_with_err;

use crate::cell_insertion::insert_vertices_on_edge;
use crate::grisubal::model::{Boundary, MapEdge};

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
        cmap.force_unlink::<1>(*start).unwrap();
        cmap.force_unlink::<1>(b0_end_old).unwrap();

        let &[d_new, b2_d_new] = &dslice[0..2] else {
            unreachable!()
        };
        cmap.force_link::<2>(d_new, b2_d_new).unwrap();

        // rebuild - this is the final construct if there are no intermediates
        cmap.force_link::<1>(*start, d_new).unwrap();
        cmap.force_link::<1>(b2_d_new, b1_start_old).unwrap();
        cmap.force_link::<1>(d_new, *end).unwrap();
        cmap.force_link::<1>(b0_end_old, b2_d_new).unwrap();

        if !intermediates.is_empty() {
            // create the topology components
            let edge_id = cmap.edge_id(d_new);
            let new_darts = &dslice[2..];
            atomically_with_err(|trans| {
                insert_vertices_on_edge(
                    cmap,
                    trans,
                    edge_id,
                    new_darts,
                    &vec![T::from(0.5).unwrap(); intermediates.len()],
                )
            })
            .unwrap();
            // replace placeholder vertices
            let mut dart_id = cmap.beta::<1>(edge_id as DartIdType);
            for v in intermediates {
                let vid = cmap.vertex_id(dart_id);
                let _ = cmap.force_write_vertex(vid, *v);
                dart_id = cmap.beta::<1>(dart_id);
            }
        }

        let mut d_boundary = cmap.beta::<1>(*start);
        while d_boundary != *end {
            cmap.force_write_attribute::<Boundary>(d_boundary, Boundary::Left);
            cmap.force_write_attribute::<Boundary>(cmap.beta::<2>(d_boundary), Boundary::Right);
            d_boundary = cmap.beta::<1>(d_boundary);
        }
    }
}
