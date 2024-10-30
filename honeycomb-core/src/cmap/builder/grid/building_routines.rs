//! Internal grid-building routines

// ------ IMPORTS

use crate::prelude::{CMap2, DartIdentifier, Vector2, Vertex2};
use crate::{attributes::AttrStorageManager, geometry::CoordsFloat};

// ------ CONTENT

/// Internal grid-building routine
#[allow(clippy::too_many_lines)]
pub fn build_2d_grid<T: CoordsFloat>(
    origin: Vertex2<T>,
    [n_square_x, n_square_y]: [usize; 2],
    [len_per_x, len_per_y]: [T; 2],
    manager: AttrStorageManager,
) -> CMap2<T> {
    let map: CMap2<T> = CMap2::new_with_undefined_attributes(4 * n_square_x * n_square_y, manager);

    // regular iterations (skip last row/col)
    (0..n_square_y - 1)
        // flatten the loop to expose more parallelism
        .flat_map(|y_idx| (0..n_square_x - 1).map(move |x_idx| (y_idx, x_idx)))
        .for_each(|(y_idx, x_idx)| {
            // build basic topology & fetch dart IDs of the cell
            let [d1, d2, d3, _] = build_square_core(&mut map, n_square_x, [x_idx, y_idx]);

            // sew to right & up neighbors
            build_square_sew_right(&mut map, d2);
            build_square_sew_up(&mut map, d3, n_square_x);

            // edit geometry
            let vertex_id = map.vertex_id(d1); // bottom left
            map.insert_vertex(
                vertex_id,
                origin
                    + Vector2(
                        T::from(x_idx).unwrap() * len_per_x,
                        T::from(y_idx).unwrap() * len_per_y,
                    ),
            );
        });

    // last row (except top right square)
    (0..n_square_x - 1).for_each(|x_idx| {
        let y_idx = n_square_y - 1;

        // build basic topology & fetch dart IDs of the cell
        let [d1, d2, _, d4] = build_square_core(&mut map, n_square_x, [x_idx, y_idx]);

        // sew to right neighbor only
        build_square_sew_right(&mut map, d2);

        // edit geometry
        let vertex_id = map.vertex_id(d1); // bottom left
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d4); // top left
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx + 1).unwrap() * len_per_y,
                ),
        );
    });

    // last col (except top right square)
    (0..n_square_y - 1).for_each(|y_idx| {
        let x_idx = n_square_x - 1;

        // build basic topology & fetch dart IDs of the cell
        let [d1, d2, d3, _] = build_square_core(&mut map, n_square_x, [x_idx, y_idx]);

        // sew to up neighbor only
        build_square_sew_up(&mut map, d3, n_square_x);

        // edit geometry
        let vertex_id = map.vertex_id(d1); // bottom left
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d2); // bottom right
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx + 1).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
    });

    // most top right cell
    {
        let (x_idx, y_idx) = (n_square_x - 1, n_square_y - 1);

        // build basic topology & fetch dart IDs of the cell
        let [d1, d2, d3, d4] = build_square_core(&mut map, n_square_x, [x_idx, y_idx]);

        // edit geometry
        let vertex_id = map.vertex_id(d1); // bottom left
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d2); // bottom right
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx + 1).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d4); // top left
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx + 1).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d3); // top right
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx + 1).unwrap() * len_per_x,
                    T::from(y_idx + 1).unwrap() * len_per_y,
                ),
        );
    }

    // check the number of built faces
    // this is set as debug only because the operation cost scales with map size
    // this can quickly overshadow the exectime of all previous code
    debug_assert_eq!(map.fetch_faces().identifiers.len(), n_square_x * n_square_y);

    map
}

#[allow(clippy::inline_always)]
#[inline(always)] // seems like this is required to match the actual inline perf
fn build_square_core<T: CoordsFloat>(
    map: &mut CMap2<T>,
    n_square_x: usize,
    [x_idx, y_idx]: [usize; 2],
) -> [DartIdentifier; 4] {
    let d1 = (1 + 4 * x_idx + n_square_x * 4 * y_idx) as DartIdentifier;
    let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);

    // edit topology
    // d1
    map.set_beta::<0>(d1, d4);
    map.set_beta::<1>(d1, d2);
    // d2
    map.set_beta::<0>(d2, d1);
    map.set_beta::<1>(d2, d3);
    // d3
    map.set_beta::<0>(d3, d2);
    map.set_beta::<1>(d3, d4);
    // d4
    map.set_beta::<0>(d4, d3);
    map.set_beta::<1>(d4, d1);

    [d1, d2, d3, d4]
}

#[allow(clippy::inline_always)]
#[inline(always)] // seems like this is required to match the actual inline perf
fn build_square_sew_right<T: CoordsFloat>(map: &mut CMap2<T>, dart: DartIdentifier) {
    let right_neighbor = dart + 6;
    map.set_beta::<2>(dart, right_neighbor);
    map.set_beta::<2>(right_neighbor, dart);
}

#[allow(clippy::inline_always)]
#[inline(always)] // seems like this is required to match the actual inline perf
fn build_square_sew_up<T: CoordsFloat>(
    map: &mut CMap2<T>,
    dart: DartIdentifier,
    n_square_x: usize,
) {
    let up_neighbor = dart - 2 + 4 * n_square_x as DartIdentifier; // d1 + 4*nx
    map.set_beta::<2>(dart, up_neighbor);
    map.set_beta::<2>(up_neighbor, dart);
}

/// Internal grid-building routine
#[allow(clippy::too_many_lines)]
pub fn build_2d_splitgrid<T: CoordsFloat>(
    origin: Vertex2<T>,
    [n_square_x, n_square_y]: [usize; 2],
    [len_per_x, len_per_y]: [T; 2],
    manager: AttrStorageManager,
) -> CMap2<T> {
    let map: CMap2<T> = CMap2::new_with_undefined_attributes(6 * n_square_x * n_square_y, manager);

    (0..n_square_y - 1)
        // flatten the loop to expose more parallelism
        // this is not a quantified/benchmarked upgrade, just a seemingly good change
        .flat_map(|y_idx| (0..n_square_x - 1).map(move |x_idx| (y_idx, x_idx)))
        .for_each(|(y_idx, x_idx)| {
            // build basic topology & fetch dart IDs of the cell
            let [d1, _, _, _, d5, d6] = build_tris_core(&mut map, n_square_x, [x_idx, y_idx]);

            // if there is a right neighbor, sew sew
            build_tris_sew_right(&mut map, d5);
            // if there is an up neighbor, sew sew
            build_tris_sew_up(&mut map, d6, n_square_x);

            // edit geometry
            let vertex_id = map.vertex_id(d1);
            map.insert_vertex(
                vertex_id,
                origin
                    + Vector2(
                        T::from(x_idx).unwrap() * len_per_x,
                        T::from(y_idx).unwrap() * len_per_y,
                    ),
            );
        });

    // last row (except top right square)
    (0..n_square_x - 1).for_each(|x_idx| {
        let y_idx = n_square_y - 1;

        // build basic topology & fetch dart IDs of the cell
        let [d1, _, d3, _, d5, _] = build_tris_core(&mut map, n_square_x, [x_idx, y_idx]);

        // sew right neighbor
        build_tris_sew_right(&mut map, d5);

        // edit geometry
        let vertex_id = map.vertex_id(d1);
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d3);
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx + 1).unwrap() * len_per_y,
                ),
        );
    });

    // last col (except top right square)
    (0..n_square_y - 1).for_each(|y_idx| {
        let x_idx = n_square_x - 1;

        // build basic topology & fetch dart IDs of the cell
        let [d1, _, _, _, d5, d6] = build_tris_core(&mut map, n_square_x, [x_idx, y_idx]);

        // sew up neighbor
        build_tris_sew_up(&mut map, d6, n_square_x);

        // edit geometry
        let vertex_id = map.vertex_id(d1);
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d5);
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx + 1).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
    });

    // most top right cell
    {
        let (x_idx, y_idx) = (n_square_x - 1, n_square_y - 1);

        // build basic topology & fetch dart IDs of the cell
        let [d1, _, d3, _, d5, d6] = build_tris_core(&mut map, n_square_x, [x_idx, y_idx]);

        let vertex_id = map.vertex_id(d1);
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d3);
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx + 1).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d5);
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx + 1).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d6);
        map.insert_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx + 1).unwrap() * len_per_x,
                    T::from(y_idx + 1).unwrap() * len_per_y,
                ),
        );
    }

    // check the number of built faces
    // this is set as debug only because the operation cost scales with map size
    // this can quickly overshadow the exectime of all previous code
    debug_assert_eq!(
        map.fetch_faces().identifiers.len(),
        n_square_x * n_square_y * 2
    );

    map
}

#[allow(clippy::inline_always)]
#[inline(always)] // seems like this is required to match the actual inline perf
fn build_tris_core<T: CoordsFloat>(
    map: &mut CMap2<T>,
    n_square_x: usize,
    [x_idx, y_idx]: [usize; 2],
) -> [DartIdentifier; 6] {
    let d1 = (1 + 6 * (x_idx + n_square_x * y_idx)) as DartIdentifier;
    let (d2, d3, d4, d5, d6) = (d1 + 1, d1 + 2, d1 + 3, d1 + 4, d1 + 5);

    // edit topology
    // d1
    map.set_beta::<0>(d1, d3);
    map.set_beta::<1>(d1, d2);
    // d2
    map.set_beta::<0>(d2, d1);
    map.set_beta::<1>(d2, d3);
    // d3
    map.set_beta::<0>(d3, d2);
    map.set_beta::<1>(d3, d1);
    // d4
    map.set_beta::<0>(d4, d6);
    map.set_beta::<1>(d4, d5);
    // d5
    map.set_beta::<0>(d5, d4);
    map.set_beta::<1>(d5, d6);
    // d6
    map.set_beta::<0>(d6, d5);
    map.set_beta::<1>(d6, d4);
    // diagonal
    map.set_beta::<2>(d2, d4);
    map.set_beta::<2>(d4, d2);

    [d1, d2, d3, d4, d5, d6]
}

#[allow(clippy::inline_always)]
#[inline(always)] // seems like this is required to match the actual inline perf
fn build_tris_sew_right<T: CoordsFloat>(map: &mut CMap2<T>, dart: DartIdentifier) {
    let right_neighbor = dart + 4;
    map.set_beta::<2>(dart, right_neighbor);
    map.set_beta::<2>(right_neighbor, dart);
}

#[allow(clippy::inline_always)]
#[inline(always)] // seems like this is required to match the actual inline perf
fn build_tris_sew_up<T: CoordsFloat>(map: &mut CMap2<T>, dart: DartIdentifier, n_square_x: usize) {
    let up_neighbor = dart - 5 + 6 * n_square_x as DartIdentifier; // d1 + 6*nx
    map.set_beta::<2>(dart, up_neighbor);
    map.set_beta::<2>(up_neighbor, dart);
}
