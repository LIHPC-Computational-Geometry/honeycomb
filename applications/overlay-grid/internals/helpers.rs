use crate::internals::model::IsIrregular;
use honeycomb::{
    core::{
        cmap::{CMap2, NULL_DART_ID},
        geometry::{CoordsFloat, Vertex2},
    },
    prelude::OrbitPolicy,
    stm::{StmError, Transaction},
};
use rayon::prelude::*;

pub fn dart_origin<T: CoordsFloat>(map: &CMap2<T>, dart: u32) -> Vertex2<T> {
    assert_ne!(dart, NULL_DART_ID);

    map.force_read_vertex(map.vertex_id(dart)).unwrap()
}

pub fn canonical_beta1_tx<T: CoordsFloat>(
    trans: &mut Transaction,
    map: &CMap2<T>,
    dart: u32,
) -> Result<(u32, u32), StmError> {
    //assert_ne!(dart, NULL_DART_ID);

    let mut steps = 1;
    let mut current_dart = map.beta_tx::<1>(trans, dart)?;

    while current_dart != NULL_DART_ID && !is_regular_tx(trans, map, current_dart)? {
        current_dart = map.beta_tx::<1>(trans, current_dart)?;
        steps += 1;
    }

    Ok((steps, current_dart))
}

pub fn dart_origin_tx<T: CoordsFloat>(
    trans: &mut Transaction,
    map: &CMap2<T>,
    dart: u32,
) -> Result<Vertex2<T>, StmError> {
    //assert_ne!(dart, NULL_DART_ID);

    let v_id = map.vertex_id_tx(trans, dart)?;
    match map.read_vertex(trans, v_id)? {
        Some(vertex) => Ok(vertex),
        None => Ok(try_getting_dart_origin_from_orbit(trans, map, dart)),
    }
}

// happened because of dart "stealing"
fn try_getting_dart_origin_from_orbit<T: CoordsFloat>(
    trans: &mut Transaction,
    map: &CMap2<T>,
    dart: u32,
) -> Vertex2<T> {
    let mut found_vertex = Vertex2::<T>::from((T::zero(), T::zero()));
    // Collect the orbit darts first to avoid borrowing issues
    let orbit_darts: Vec<u32> = map
        .orbit_tx(trans, OrbitPolicy::Vertex, dart)
        .filter_map(|dart_result| dart_result.ok())
        .collect();

    for ve in orbit_darts {
        if let Ok(Some(v)) = map.read_vertex(trans, ve) {
            found_vertex = v;
            break;
        }
    }
    found_vertex
}

pub fn collect_face_darts_tx<T: CoordsFloat>(
    trans: &mut Transaction,
    map: &CMap2<T>,
    start_dart: u32,
) -> Result<Vec<u32>, StmError> {
    let mut face_darts = Vec::new();

    // We want sure that our structure starts with a regular dart
    let mut current_dart = start_dart;
    while !is_regular_tx(trans, map, current_dart)? {
        current_dart = map.beta_tx::<1>(trans, current_dart)?;
    }

    // then after we retrieve only regular darts
    let new_start = current_dart;
    face_darts.push(new_start);

    current_dart = map.beta_tx::<1>(trans, current_dart)?;
    while current_dart != new_start {
        if is_regular_tx(trans, map, current_dart)? {
            face_darts.push(current_dart);
        }
        current_dart = map.beta_tx::<1>(trans, current_dart)?;
    }

    assert_eq!(face_darts.len(), 4);

    // order the darts so the first face_dart is in SW quadrant
    let pos0 = dart_origin_tx(trans, map, face_darts[0])?;
    let pos2 = dart_origin_tx(trans, map, face_darts[2])?;
    let midpoint = Vertex2::<T>::average(&pos0, &pos2);
    let mut face_darts_ordered = vec![0u32; 4];
    for dart in face_darts.iter() {
        let pos = dart_origin_tx(trans, map, *dart)?;
        let quadrant = get_quadrant(&pos, &midpoint);
        face_darts_ordered[quadrant] = *dart;
    }

    Ok(face_darts_ordered)
}

pub fn is_regular_tx<T: CoordsFloat>(
    trans: &mut Transaction,
    map: &CMap2<T>,
    dart: u32,
) -> Result<bool, StmError> {
    Ok(!map
        .read_attribute::<IsIrregular>(trans, dart)?
        .unwrap_or(IsIrregular(false))
        .0)
}

pub fn is_regular<T: CoordsFloat>(map: &CMap2<T>, dart: u32) -> bool {
    assert_ne!(dart, NULL_DART_ID);

    let attr = map.force_read_attribute::<IsIrregular>(dart);
    attr.is_none()
}

/// Determines which quadrant a vertex belongs to relative to a midpoint
/// Returns: 0=bottom-left, 1=bottom-right, 2=top-right, 3=top-left
pub fn get_quadrant<T: CoordsFloat>(vertex: &Vertex2<T>, midpoint: &Vertex2<T>) -> usize {
    let x_right = vertex.x() >= midpoint.x();
    let y_top = vertex.y() >= midpoint.y();

    match (x_right, y_top) {
        (false, false) => 0, // bottom-left
        (true, false) => 1,  // bottom-right
        (true, true) => 2,   // top-right
        (false, true) => 3,  // top-left
    }
}

pub fn remove_dangling_darts<T: CoordsFloat>(map: &mut CMap2<T>) {
    map.par_iter_vertices().for_each(|vertex_id| {
        let vertex = map.force_read_vertex(map.vertex_id(vertex_id));
        if vertex.is_none() {
            // look for a dart in the orbit that actually has a vertex.
            // it's probably been "stolen" when a dart with a small id has been linked
            map.i_cell::<0>(vertex_id).for_each(|d| {
                let v = map.force_read_vertex(d);
                if let Some(vertex) = v {
                    map.force_write_vertex(vertex_id, vertex);
                }
            });
        }
    });
}
