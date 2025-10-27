use honeycomb::{
    prelude::{
        CMap2, CoordsFloat, DartIdType, OrbitPolicy,
        triangulation::{earclip_cell_countercw, fan_cell},
    },
    stm::atomically_with_err,
};

pub fn earclip_cells<T: CoordsFloat>(map: &mut CMap2<T>) {
    let faces: Vec<_> = map.iter_faces().collect();
    let n_darts_per_face: Vec<_> = faces
        .iter()
        .map(|id| (map.orbit(OrbitPolicy::Face, *id as DartIdType).count() - 3) * 2)
        .collect();
    let n_tot: usize = n_darts_per_face.iter().sum();
    let tmp = map.allocate_used_darts(n_tot) as usize;
    // the prefix sum gives an offset that corresponds to the starting index of each slice, minus
    // the location of the allocated dart block (given by `tmp`)
    // end of the slice is deduced using these values and the number of darts the current seg needs
    let prefix_sum = n_darts_per_face.iter().scan(0, |state, &n_d| {
        *state += n_d;
        Some(*state - n_d) // we want an offset, not the actual sum
    });
    #[allow(clippy::cast_possible_truncation)]
    let dart_slices: Vec<Vec<DartIdType>> = n_darts_per_face
        .iter()
        .zip(prefix_sum)
        .map(|(n_d, start)| {
            ((tmp + start) as DartIdType..(tmp + start + n_d) as DartIdType).collect::<Vec<_>>()
        })
        .collect();

    for (face_id, new_darts) in faces.iter().zip(dart_slices.iter()) {
        let _ = atomically_with_err(|t| earclip_cell_countercw(t, map, *face_id, new_darts));
    }
}

pub fn fan_cells<T: CoordsFloat>(map: &mut CMap2<T>) {
    let faces: Vec<_> = map.iter_faces().collect();
    let n_darts_per_face: Vec<_> = faces
        .iter()
        .map(|id| (map.orbit(OrbitPolicy::Face, *id as DartIdType).count() - 3) * 2)
        .collect();
    let n_tot: usize = n_darts_per_face.iter().sum();
    let tmp = map.allocate_used_darts(n_tot) as usize;
    // the prefix sum gives an offset that corresponds to the starting index of each slice, minus
    // the location of the allocated dart block (given by `tmp`)
    // end of the slice is deduced using these values and the number of darts the current seg needs
    let prefix_sum = n_darts_per_face.iter().scan(0, |state, &n_d| {
        *state += n_d;
        Some(*state - n_d) // we want an offset, not the actual sum
    });
    #[allow(clippy::cast_possible_truncation)]
    let dart_slices: Vec<Vec<DartIdType>> = n_darts_per_face
        .iter()
        .zip(prefix_sum)
        .map(|(n_d, start)| {
            ((tmp + start) as DartIdType..(tmp + start + n_d) as DartIdType).collect::<Vec<_>>()
        })
        .collect();

    for (face_id, new_darts) in faces.iter().zip(dart_slices.iter()) {
        let _ = atomically_with_err(|t| fan_cell(t, map, *face_id, new_darts));
    }
}
