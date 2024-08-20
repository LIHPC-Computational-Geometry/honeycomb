//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::grisubal::model::Boundary;
use crate::GrisubalError;
use crate::GrisubalError::InconsistentOrientation;
use honeycomb_core::{
    CMap2, CoordsFloat, DartIdentifier, FaceIdentifier, Orbit2, OrbitPolicy, NULL_DART_ID,
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
                return Err(InconsistentOrientation(
                    format!("reached right side (dart #{rid}, face #{face_id}) without crossing the boundary")
                ));
            }
            // find neighbor faces where darts aren't tagged
            queue.extend(darts.filter_map(|dart_id| {
                if matches!(
                    cmap.get_attribute::<Boundary>(cmap.beta::<2>(dart_id)),
                    Some(Boundary::None)
                ) {
                    return Some(cmap.face_id(cmap.beta::<2>(dart_id)));
                }
                None
            }));
        }
    }

    // split the boundary & nuke all darts of marked faces
    for &face_id in &marked {
        let darts: Vec<DartIdentifier> =
            Orbit2::new(&cmap, OrbitPolicy::Face, face_id as DartIdentifier).collect();
        for &dart in &darts {
            if cmap.beta::<2>(dart) != NULL_DART_ID {
                cmap.two_unsew(dart);
            }
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
    (1..cmap.n_darts() as DartIdentifier).for_each(|dart_id| {
        // use darts on the left side of the boundary as starting points to walk through faces
        if matches!(
            cmap.get_attribute::<Boundary>(dart_id),
            Some(Boundary::Right),
        ) {
            println!("vertex: {:#?}", cmap.vertex(cmap.vertex_id(dart_id)))
        }
    });

    Ok(cmap)
}
