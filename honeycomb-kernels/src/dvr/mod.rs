//! Directional Vertex Relaxation implementation

// ------ MODULE DECLARATIONS

mod maximizers;
mod quality;

// ------ IMPORTS

use honeycomb_core::{
    cmap::{CMap2, DartIdentifier, Orbit2, OrbitPolicy, VertexIdentifier},
    prelude::{CoordsFloat, Vector2},
};

// ------ CONTENT

/// Regular DVR implementation.
#[allow(clippy::needless_for_each)]
pub fn dvr<T: CoordsFloat>(
    map: &mut CMap2<T>,
    n_relaxations: usize,
    dirs_relaxations: Option<Vec<Vector2<T>>>,
) {
    // DVR does not affect topology, so these IDs will stay valid during the entire run
    // we can filter-out vertices on the boundary from the get-go
    let vertices: Vec<VertexIdentifier> = map
        .fetch_vertices()
        .identifiers
        .iter()
        .filter(|vid| {
            !Orbit2::new(map, OrbitPolicy::Vertex, **vid as DartIdentifier)
                .any(|dart_id| map.is_i_free::<2>(dart_id))
        })
        .copied()
        .collect();

    // use arg relaxation dirs if passed, else default to cartesian dirs
    let dirs = dirs_relaxations.unwrap_or(vec![Vector2::unit_x(), Vector2::unit_y()]);

    // using a for outside since this should not be parallelized
    // iterator inside since it can be with changes to vertex selection
    for k in 0..n_relaxations {
        let _dir = dirs[k % dirs.len()];
        vertices.iter().for_each(|_vid| {
            // compute the set of quality maximizers
            // if card == 1 => set lambda_opt
            // else find lambda_opt s.t. ...
            // v <= v + dir * lambda_opt
            todo!()
        });
    }
}

/// Approximate DVR implementation.
#[allow(clippy::needless_for_each)]
pub fn dvr_approximate<T: CoordsFloat>(
    map: &mut CMap2<T>,
    n_relaxations: usize,
    dirs_relaxations: Option<Vec<Vector2<T>>>,
) {
    // DVR does not affect topology, so these IDs will stay valid during the entire run
    // we can filter-out vertices on the boundary from the get-go
    let vertices: Vec<VertexIdentifier> = map
        .fetch_vertices()
        .identifiers
        .iter()
        .filter(|vid| {
            !Orbit2::new(map, OrbitPolicy::Vertex, **vid as DartIdentifier)
                .any(|dart_id| map.is_i_free::<2>(dart_id))
        })
        .copied()
        .collect();

    // use arg relaxation dirs if passed, else default to cartesian dirs
    let dirs = dirs_relaxations.unwrap_or(vec![Vector2::unit_x(), Vector2::unit_y()]);

    // using a for outside since this should not be parallelized
    // iterator inside since it can be with changes to vertex selection
    for k in 0..n_relaxations {
        let _dir = dirs[k % dirs.len()];
        vertices.iter().for_each(|_vid| {
            // find lambda s.t. quality(lambda) > quality(0)
            // v <= v + dir * lambda
            todo!()
        });
    }
}
