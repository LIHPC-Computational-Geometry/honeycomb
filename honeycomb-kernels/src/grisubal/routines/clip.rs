//! clipping operation routines

// ------ IMPORTS

use crate::grisubal::model::Boundary;
use crate::grisubal::GrisubalError;
use honeycomb_core::{
    cmap::{DartIdType, NULL_FACE_ID},
    prelude::{CMap2, CoordsFloat, DartId, FaceId, Orbit2, OrbitPolicy, Vertex2, NULL_DART_ID},
};
use std::collections::{HashSet, VecDeque};

// ------ CONTENT

/// Clip content on the left side of the boundary.
pub fn clip_left<T: CoordsFloat>(cmap: &mut CMap2<T>) -> Result<(), GrisubalError> {
    // color faces using a bfs starting on multiple nodes
    let marked = mark_faces(cmap, Boundary::Left, Boundary::Right)?;

    // save vertices & split boundary
    delete_darts(cmap, marked, Boundary::Right);

    Ok(())
}

/// Clip content on the right side of the boundary.
pub fn clip_right<T: CoordsFloat>(cmap: &mut CMap2<T>) -> Result<(), GrisubalError> {
    // color faces using a bfs starting on multiple nodes
    let marked = mark_faces(cmap, Boundary::Right, Boundary::Left)?;

    // save vertices & split boundary
    delete_darts(cmap, marked, Boundary::Left);

    Ok(())
}

// --- internals

#[allow(clippy::cast_possible_truncation)]
fn mark_faces<T: CoordsFloat>(
    cmap: &CMap2<T>,
    mark: Boundary,
    other: Boundary,
) -> Result<HashSet<FaceId>, GrisubalError> {
    let mut marked: HashSet<FaceId> = HashSet::from([NULL_FACE_ID]);
    let mut queue: VecDeque<FaceId> = (1..cmap.n_darts() as DartIdType)
        .map(DartId)
        .filter_map(|dart_id| {
            // use darts on the left side of the boundary as starting points to walk through faces
            if cmap.get_attribute::<Boundary>(dart_id) == Some(mark) && !cmap.is_free(dart_id) {
                return Some(cmap.face_id(dart_id));
            }
            None
        })
        .collect();
    while let Some(face_id) = queue.pop_front() {
        // mark faces
        if marked.insert(face_id) {
            // detect orientation issues / open geometries
            let mut darts = Orbit2::new(cmap, OrbitPolicy::Face, face_id);
            if darts.any(|did| cmap.get_attribute::<Boundary>(did) == Some(other)) {
                return Err(GrisubalError::InconsistentOrientation(
                    "between-boundary inconsistency",
                ));
            }
            // find neighbor faces where entry darts aren't tagged
            let darts = Orbit2::new(cmap, OrbitPolicy::Face, face_id);
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
    marked.remove(&NULL_FACE_ID);
    Ok(marked)
}

#[allow(clippy::cast_possible_truncation)]
fn delete_darts<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    marked: HashSet<FaceId>,
    kept_boundary: Boundary,
) {
    let kept_boundary_components: Vec<(DartId, Vertex2<T>)> = (1..cmap.n_darts() as DartIdType)
        .map(DartId)
        .filter_map(|dart_id| {
            if cmap.get_attribute::<Boundary>(dart_id) == Some(kept_boundary) {
                return Some((
                    dart_id,
                    cmap.vertex(cmap.vertex_id(dart_id))
                        .expect("E: found a topological vertex with no associated coordinates"),
                ));
            }
            None
        })
        .collect();

    for face_id in marked {
        let darts: Vec<DartId> = Orbit2::new(cmap, OrbitPolicy::Face, face_id).collect();
        for &dart in &darts {
            let _ = cmap.remove_vertex(cmap.vertex_id(dart));
            cmap.set_betas(dart, [NULL_DART_ID; 3]);
            cmap.remove_free_dart(dart);
        }
    }

    for (dart, vertex) in kept_boundary_components {
        cmap.set_beta::<2>(dart, NULL_DART_ID); // set beta2(dart) to 0
        cmap.insert_vertex(cmap.vertex_id(dart), vertex);
    }
}
