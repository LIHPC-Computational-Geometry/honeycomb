//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::grisubal::model::Boundary;
use crate::GrisubalError;
use honeycomb_core::{
    CMap2, CoordsFloat, DartIdentifier, FaceIdentifier, Orbit2, OrbitPolicy, Vertex2, NULL_DART_ID,
};
use std::collections::{HashSet, VecDeque};
// ------ CONTENT

pub fn clip_left<T: CoordsFloat>(mut cmap: CMap2<T>) -> Result<CMap2<T>, GrisubalError> {
    // color faces using a bfs starting on multiple nodes
    let left_bound: Vec<DartIdentifier> = (1..cmap.n_darts() as DartIdentifier)
        .filter_map(|dart_id| {
            // use darts on the left side of the boundary as starting points to walk through faces
            if matches!(
                cmap.get_attribute::<Boundary>(dart_id),
                Some(Boundary::Left),
            ) && !cmap.is_free(dart_id)
            {
                return Some(cmap.face_id(dart_id));
            }
            None
        })
        .collect();
    let mut marked: HashSet<FaceIdentifier> = HashSet::from([0]);
    let mut queue: VecDeque<FaceIdentifier> = VecDeque::from(left_bound.clone());
    while let Some(face_id) = queue.pop_front() {
        // mark faces
        if marked.insert(face_id) {
            // detect orientation issues / open geometries
            let mut darts = Orbit2::new(&cmap, OrbitPolicy::Face, face_id as DartIdentifier);
            if let Some(rid) = darts
                .find(|did| matches!(cmap.get_attribute::<Boundary>(*did), Some(Boundary::Right)))
            {
                // TODO: explain why it is an inconsistency
                return Err(GrisubalError::InconsistentOrientation(
                    format!("reached right side (dart #{rid}, face #{face_id}) without crossing the boundary")
                ));
            }
            // find neighbor faces where entry darts aren't tagged
            let darts = Orbit2::new(&cmap, OrbitPolicy::Face, face_id as DartIdentifier);
            queue.extend(darts.filter_map(|dart_id| {
                if matches!(
                    cmap.get_attribute::<Boundary>(cmap.beta::<2>(dart_id)),
                    Some(Boundary::None) | None
                ) {
                    return Some(cmap.face_id(cmap.beta::<2>(dart_id)));
                }
                None
            }));
        }
    }
    marked.remove(&0);

    // save vertices & split boundary
    let right_boundary: Vec<(DartIdentifier, Vertex2<T>)> = (1..cmap.n_darts() as DartIdentifier)
        .filter_map(|dart_id| {
            if matches!(
                cmap.get_attribute::<Boundary>(dart_id),
                Some(Boundary::Right),
            ) {
                return Some((dart_id, cmap.vertex(cmap.vertex_id(dart_id)).unwrap()));
            }
            None
        })
        .collect();

    for &face_id in &marked {
        let darts: Vec<DartIdentifier> =
            Orbit2::new(&cmap, OrbitPolicy::Face, face_id as DartIdentifier).collect();
        for &dart in &darts {
            let _ = cmap.remove_vertex(cmap.vertex_id(dart));
        }
    }

    for &face_id in &marked {
        let darts: Vec<DartIdentifier> =
            Orbit2::new(&cmap, OrbitPolicy::Face, face_id as DartIdentifier).collect();
        for &dart in &darts {
            cmap.set_betas(dart, [NULL_DART_ID; 3]);
            cmap.remove_free_dart(dart);
        }
    }
    for (dart, vertex) in right_boundary {
        let b1 = cmap.beta::<1>(dart);
        let b0 = cmap.beta::<0>(dart);
        cmap.set_betas(dart, [b0, b1, NULL_DART_ID]); // set beta2(dart) to 0
        cmap.insert_vertex(cmap.vertex_id(dart), vertex);
    }

    Ok(cmap)
}
