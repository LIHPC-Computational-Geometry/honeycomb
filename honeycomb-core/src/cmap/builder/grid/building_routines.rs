//! Internal grid-building routines

// ------ IMPORTS

use crate::prelude::{CMap2, DartIdType, Vector2, Vertex2};
use crate::{attributes::AttrStorageManager, geometry::CoordsFloat};

// ------ CONTENT

/// Internal grid-building routine
#[allow(clippy::too_many_lines)]
pub fn build_2d_grid_alt<T: CoordsFloat>(
    origin: Vertex2<T>,
    [n_square_x, n_square_y]: [usize; 2],
    [len_per_x, len_per_y]: [T; 2],
    manager: AttrStorageManager,
) -> CMap2<T> {
    let map: CMap2<T> = CMap2::new_with_undefined_attributes(4 * n_square_x * n_square_y, manager);

    // init beta functions
    (1..=(4 * n_square_x * n_square_y) as DartIdType)
        .zip(generate_beta_values(n_square_x, n_square_y))
        .for_each(|(dart, images)| {
            map.set_betas(dart, images);
        });

    // place vertices

    // bottow left vertex of all cells
    (0..n_square_y)
        // flatten the loop to expose more parallelism
        .flat_map(|y_idx| (0..n_square_x).map(move |x_idx| (y_idx, x_idx)))
        .for_each(|(y_idx, x_idx)| {
            let vertex_id = map.vertex_id((1 + x_idx * 4 + y_idx * 4 * n_square_x) as DartIdType);
            map.force_write_vertex(
                vertex_id,
                origin
                    + Vector2(
                        T::from(x_idx).unwrap() * len_per_x,
                        T::from(y_idx).unwrap() * len_per_y,
                    ),
            );
        });

    // top left vertex of all top row cells
    (0..n_square_x).for_each(|x_idx| {
        let y_idx = n_square_y - 1;
        let vertex_id = map.vertex_id((4 + x_idx * 4 + y_idx * 4 * n_square_x) as DartIdType);
        map.force_write_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx + 1).unwrap() * len_per_y,
                ),
        );
    });

    // bottom right vertex of all right col cells
    (0..n_square_y).for_each(|y_idx| {
        let x_idx = n_square_x - 1;
        let vertex_id = map.vertex_id((2 + x_idx * 4 + y_idx * 4 * n_square_x) as DartIdType);
        map.force_write_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx + 1).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
    });

    // top right vertex of the last cell
    {
        let (x_idx, y_idx) = (n_square_x - 1, n_square_y - 1);
        let vertex_id = map.vertex_id((3 + x_idx * 4 + y_idx * 4 * n_square_x) as DartIdType); // top right
        map.force_write_vertex(
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
#[rustfmt::skip]
#[inline(always)]
fn generate_beta_values(n_x: usize, n_y: usize) -> impl Iterator<Item = [DartIdType; 3]> {
    // this loop hierarchy yields the value in correct order
    (0..n_y).flat_map(move |iy| {
        (0..n_x).flat_map(move |ix| {
                let d1 = (1 + 4 * ix + n_x * 4 * iy) as DartIdType;
                let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
                // beta images of [d1, d2, d3, d4]
                [
                    [ d4, d2, if iy == 0     { 0 } else { d3 - 4 * n_x as DartIdType } ],
                    [ d1, d3, if ix == n_x-1 { 0 } else { d2 + 6                     } ],
                    [ d2, d4, if iy == n_y-1 { 0 } else { d1 + 4 * n_x as DartIdType } ],
                    [ d3, d1, if ix == 0     { 0 } else { d4 - 6                     } ],
                ]
                .into_iter()
            })
        })
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
            let [d1, _, _, _, d5, d6] = build_tris_core(&map, n_square_x, [x_idx, y_idx]);

            // if there is a right neighbor, sew sew
            build_tris_sew_right(&map, d5);
            // if there is an up neighbor, sew sew
            build_tris_sew_up(&map, d6, n_square_x);

            // edit geometry
            let vertex_id = map.vertex_id(d1);
            map.force_write_vertex(
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
        let [d1, _, d3, _, d5, _] = build_tris_core(&map, n_square_x, [x_idx, y_idx]);

        // sew right neighbor
        build_tris_sew_right(&map, d5);

        // edit geometry
        let vertex_id = map.vertex_id(d1);
        map.force_write_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d3);
        map.force_write_vertex(
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
        let [d1, _, _, _, d5, d6] = build_tris_core(&map, n_square_x, [x_idx, y_idx]);

        // sew up neighbor
        build_tris_sew_up(&map, d6, n_square_x);

        // edit geometry
        let vertex_id = map.vertex_id(d1);
        map.force_write_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d5);
        map.force_write_vertex(
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
        let [d1, _, d3, _, d5, d6] = build_tris_core(&map, n_square_x, [x_idx, y_idx]);

        let vertex_id = map.vertex_id(d1);
        map.force_write_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d3);
        map.force_write_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx).unwrap() * len_per_x,
                    T::from(y_idx + 1).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d5);
        map.force_write_vertex(
            vertex_id,
            origin
                + Vector2(
                    T::from(x_idx + 1).unwrap() * len_per_x,
                    T::from(y_idx).unwrap() * len_per_y,
                ),
        );
        let vertex_id = map.vertex_id(d6);
        map.force_write_vertex(
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
    map: &CMap2<T>,
    n_square_x: usize,
    [x_idx, y_idx]: [usize; 2],
) -> [DartIdType; 6] {
    let d1 = (1 + 6 * (x_idx + n_square_x * y_idx)) as DartIdType;
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
fn build_tris_sew_right<T: CoordsFloat>(map: &CMap2<T>, dart: DartIdType) {
    let right_neighbor = dart + 4;
    map.set_beta::<2>(dart, right_neighbor);
    map.set_beta::<2>(right_neighbor, dart);
}

#[allow(clippy::inline_always)]
#[inline(always)] // seems like this is required to match the actual inline perf
fn build_tris_sew_up<T: CoordsFloat>(map: &CMap2<T>, dart: DartIdType, n_square_x: usize) {
    let up_neighbor = dart - 5 + 6 * n_square_x as DartIdType; // d1 + 6*nx
    map.set_beta::<2>(dart, up_neighbor);
    map.set_beta::<2>(up_neighbor, dart);
}
