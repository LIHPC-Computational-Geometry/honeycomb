use honeycomb::core::{
    cmap::{CMap2, NULL_DART_ID},
    geometry::{CoordsFloat, Vertex2},
};
use rayon::prelude::*;
use rustc_hash::FxHashSet as HashSet;

use crate::internals::{
    helpers::{dart_origin, is_regular},
    model::{IsIrregular, RefinementLevel},
};

pub fn regularize_map<T: CoordsFloat>(map: &mut CMap2<T>) {
    let irregular_darts: Vec<u32> = map
        .par_iter_faces()
        .flat_map(|face| {
            map.i_cell::<1>(face)
                .filter(|&dart| !is_regular(map, dart))
                .collect::<Vec<u32>>()
        })
        .collect();

    // Each pair of irregular darts are regularized with 6 new darts
    let start_dart_id = map.allocate_used_darts(irregular_darts.len() * 3);

    // Group consecutive irregular darts into chains
    let irregular_chains = find_consecutive_irregular_chains(map, &irregular_darts);

    let chains_with_dart_ids =
        irregular_chains
            .into_iter()
            .scan(start_dart_id as usize, |dart_offset, chain| {
                let current_start = *dart_offset;
                let darts_for_this_chain = (chain.len() / 2) * 6;
                *dart_offset += darts_for_this_chain;
                Some((chain, current_start as u32))
            });

    let par_chains_with_dart_ids = chains_with_dart_ids.collect::<Vec<_>>().into_par_iter();

    // Process each chain with its pre-allocated dart range
    par_chains_with_dart_ids.for_each(|(chain, dart_counter)| {
        assert_eq!(chain.len() % 2, 0, "Chain length must be even");
        regularize_consecutive_chain(map, &chain, dart_counter);
    });
}

fn regularize_edge<T: CoordsFloat>(map: &CMap2<T>, edge_dart: u32, dart1: u32) -> u32 {
    let next = map.beta::<1>(edge_dart);
    let next_next = map.beta::<1>(next);
    let prev = map.beta::<0>(edge_dart);
    let next_opposite = map.beta::<2>(next);
    let prev2 = map.beta::<1>(next_opposite);
    let edge_dart2 = map.beta::<1>(prev2);

    assert_ne!(next_opposite, NULL_DART_ID);
    debug_assert!(!is_regular(map, edge_dart2));
    debug_assert!(!is_regular(map, edge_dart));

    map.remove_attribute::<IsIrregular>(edge_dart);
    map.remove_attribute::<IsIrregular>(edge_dart2);

    // Use pre-allocated darts instead of allocating new ones
    let dart2 = dart1 + 1;
    let tri_dart1 = dart1 + 2;
    let tri_dart2 = dart1 + 3;
    let tri_dart1_opposite = dart1 + 4;
    let tri_dart2_opposite = dart1 + 5;

    let vertex = Vertex2::<T>::average(&dart_origin(map, next_opposite), &dart_origin(map, next));
    map.write_vertex(dart1, vertex);

    map.unlink::<1>(prev).unwrap();
    map.unlink::<1>(next).unwrap();
    map.unlink::<1>(next_opposite).unwrap();
    map.unlink::<1>(prev2).unwrap();
    map.unlink::<2>(next).unwrap();

    // connecting everything in upper quad
    map.link::<1>(prev, tri_dart1_opposite).unwrap();
    map.link::<1>(tri_dart1_opposite, dart1).unwrap();
    map.link::<1>(dart1, next_next).unwrap();
    map.link::<1>(next, tri_dart1).unwrap();
    map.link::<1>(tri_dart1, edge_dart).unwrap();
    map.link::<2>(tri_dart1, tri_dart1_opposite).unwrap();

    // connecting everything in lower quad
    map.link::<1>(tri_dart2, dart2).unwrap();
    map.link::<1>(dart2, prev2).unwrap();
    map.link::<1>(prev2, tri_dart2).unwrap();
    map.link::<1>(next_opposite, tri_dart2_opposite).unwrap();
    map.link::<1>(tri_dart2_opposite, edge_dart2).unwrap();
    map.link::<2>(tri_dart2, tri_dart2_opposite).unwrap();

    map.link::<2>(next, dart2).unwrap();
    map.link::<2>(next_opposite, dart1).unwrap();

    dart1 + 6
}

/// Finds chains of consecutive irregular darts
fn find_consecutive_irregular_chains<T: CoordsFloat>(
    map: &CMap2<T>,
    irregular_darts: &[u32],
) -> Vec<Vec<u32>> {
    let mut processed = HashSet::default();
    let mut chains = Vec::new();

    for &dart in irregular_darts {
        if processed.contains(&dart) {
            continue;
        }

        // Build a chain starting from this dart
        let mut chain = vec![dart];
        processed.insert(dart);

        // Extend forward (next -> opposite -> next -> next)
        let mut current = dart;
        loop {
            let next_in_chain = find_next_consecutive_irregular(map, current);
            if let Some(next_dart) = next_in_chain {
                if irregular_darts.contains(&next_dart) && !processed.contains(&next_dart) {
                    chain.push(next_dart);
                    processed.insert(next_dart);
                    current = next_dart;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Extend backward (prev -> prev -> opposite -> prev)
        current = dart;
        let mut backward_chain = Vec::new();
        loop {
            let prev_in_chain = find_prev_consecutive_irregular(map, current);
            if let Some(prev_dart) = prev_in_chain {
                if irregular_darts.contains(&prev_dart) && !processed.contains(&prev_dart) {
                    backward_chain.push(prev_dart);
                    processed.insert(prev_dart);
                    current = prev_dart;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Combine backward and forward chains
        backward_chain.reverse();
        backward_chain.extend(chain);
        chains.push(backward_chain);
    }

    chains
}

/// Finds the next consecutive irregular dart (next -> opposite -> next -> next)
fn find_next_consecutive_irregular<T: CoordsFloat>(map: &CMap2<T>, dart: u32) -> Option<u32> {
    let next = map.beta::<1>(dart);
    let opposite = map.beta::<2>(next);
    if opposite == NULL_DART_ID {
        return None;
    }
    let opposite_next = map.beta::<1>(opposite);
    let opposite_next_next = map.beta::<1>(opposite_next);

    Some(opposite_next_next)
}

/// Finds the previous consecutive irregular dart (prev -> prev -> opposite -> prev)
fn find_prev_consecutive_irregular<T: CoordsFloat>(map: &CMap2<T>, dart: u32) -> Option<u32> {
    let prev = map.beta::<0>(dart);
    let prev_prev = map.beta::<0>(prev);

    let opposite = map.beta::<2>(prev_prev);
    if opposite == NULL_DART_ID {
        return None;
    }

    let opposite_prev = map.beta::<0>(opposite);

    Some(opposite_prev)
}

/// Regularizes a chain of consecutive irregular darts with pre-allocated dart IDs
fn regularize_consecutive_chain<T: CoordsFloat>(
    map: &CMap2<T>,
    chain: &[u32],
    dart_counter: u32,
) -> u32 {
    let mut dart_counter = dart_counter;

    // Process pairs within the chain, ensuring all darts get addressed
    for chunk in chain.chunks(2) {
        if chunk.len() == 2 {
            let dart1 = chunk[0];
            let dart2 = chunk[1];

            // Verify they form a valid pair with same refinement level
            let level1 = map.read_attribute::<RefinementLevel>(map.face_id(dart1));
            let level2 = map.read_attribute::<RefinementLevel>(map.face_id(dart2));

            if level1 == level2 {
                dart_counter = regularize_edge(map, dart1, dart_counter);
            }
        }
        assert_ne!(chunk.len(), 1, "Chunk length must not be 1");
    }
    dart_counter
}
