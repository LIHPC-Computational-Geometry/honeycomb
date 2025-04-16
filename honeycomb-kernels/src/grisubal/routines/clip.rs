//! clipping operation routines

use std::collections::{HashSet, VecDeque};

use honeycomb_core::cmap::{CMap2, DartIdType, FaceIdType, NULL_DART_ID, OrbitPolicy};
use honeycomb_core::geometry::{CoordsFloat, Vertex2};

use crate::grisubal::GrisubalError;
use crate::grisubal::model::Boundary;
use crate::remeshing::VertexAnchor;

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
) -> Result<HashSet<FaceIdType>, GrisubalError> {
    let mut marked: HashSet<FaceIdType> = HashSet::from([0]);
    let mut queue: VecDeque<FaceIdType> = (1..cmap.n_darts() as DartIdType)
        .filter_map(|dart_id| {
            // use darts on the left side of the boundary as starting points to walk through faces
            if cmap.force_read_attribute::<Boundary>(dart_id) == Some(mark)
                && !cmap.is_free(dart_id)
            {
                return Some(cmap.face_id(dart_id));
            }
            None
        })
        .collect();
    while let Some(face_id) = queue.pop_front() {
        // mark faces
        if marked.insert(face_id) {
            // detect orientation issues / open geometries
            let mut darts = cmap.orbit(OrbitPolicy::Face, face_id as DartIdType);
            if darts.any(|did| cmap.force_read_attribute::<Boundary>(did) == Some(other)) {
                return Err(GrisubalError::InconsistentOrientation(
                    "between-boundary inconsistency",
                ));
            }
            // find neighbor faces where entry darts aren't tagged
            let darts = cmap.orbit(OrbitPolicy::Face, face_id as DartIdType);
            queue.extend(darts.filter_map(|dart_id| {
                if matches!(
                    cmap.force_read_attribute::<Boundary>(cmap.beta::<2>(dart_id)),
                    Some(Boundary::None) | None
                ) {
                    return Some(cmap.face_id(cmap.beta::<2>(dart_id)));
                }
                None
            }));
        }
    }
    marked.remove(&0);
    Ok(marked)
}

#[allow(clippy::cast_possible_truncation)]
fn delete_darts<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    marked: HashSet<FaceIdType>,
    kept_boundary: Boundary,
) {
    let kept_boundary_components: Vec<(DartIdType, Vertex2<T>, Option<VertexAnchor>)> =
        (1..cmap.n_darts() as DartIdType)
            .filter_map(|dart_id| {
                if cmap.force_read_attribute::<Boundary>(dart_id) == Some(kept_boundary) {
                    return Some((
                        dart_id,
                        cmap.force_read_vertex(cmap.vertex_id(dart_id))
                            .expect("E: found a topological vertex with no associated coordinates"),
                        if cmap.contains_attribute::<VertexAnchor>() {
                            cmap.force_read_attribute(cmap.vertex_id(dart_id)) // may be Some or None
                        } else {
                            None
                        },
                    ));
                }
                None
            })
            .collect();

    for face_id in marked {
        let darts: Vec<DartIdType> = cmap
            .orbit(OrbitPolicy::Face, face_id as DartIdType)
            .collect();
        for &dart in &darts {
            let _ = cmap.force_remove_vertex(cmap.vertex_id(dart));
            cmap.set_betas(dart, [NULL_DART_ID; 3]);
            cmap.remove_free_dart(dart);
        }
    }

    for (dart, vertex, anchor) in kept_boundary_components {
        cmap.set_beta::<2>(dart, NULL_DART_ID); // set beta2(dart) to 0
        let vid = cmap.vertex_id(dart);
        cmap.force_write_vertex(vid, vertex);
        // if the map does not contain anchors, this branch is never taken
        if let Some(a) = anchor {
            cmap.force_write_attribute(vid, a);
        }
    }
}
