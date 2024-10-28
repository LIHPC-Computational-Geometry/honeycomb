//! Internal grid-building routines

// ------ IMPORTS

use crate::prelude::{CMap2, DartIdentifier, Vector2, Vertex2};
use crate::{attributes::AttrStorageManager, geometry::CoordsFloat};

// ------ CONTENT

/// Internal grid-building routine
pub fn build_2d_grid<T: CoordsFloat>(
    origin: Vertex2<T>,
    [n_square_x, n_square_y]: [usize; 2],
    [len_per_x, len_per_y]: [T; 2],
    manager: AttrStorageManager,
) -> CMap2<T> {
    let mut map: CMap2<T> =
        CMap2::new_with_undefined_attributes(4 * n_square_x * n_square_y, manager);

    (0..n_square_y)
        // flatten the loop to expose more parallelism
        // this is not a quantified/benchmarked upgrade, just a seemingly good change
        .flat_map(|y_idx| (0..n_square_x).map(move |x_idx| (y_idx, x_idx)))
        .for_each(|(y_idx, x_idx)| {
            // compute dart IDs of the cell
            let d1 = (1 + 4 * x_idx + n_square_x * 4 * y_idx) as DartIdentifier;
            let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
            // are we on the last col / row / both?
            let last_column = x_idx == n_square_x - 1;
            let last_row = y_idx == n_square_y - 1;

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
            // if there is a right neighbor, sew sew
            if !last_column {
                let right_neighbor = d2 + 6;
                map.set_beta::<2>(d2, right_neighbor);
                map.set_beta::<2>(right_neighbor, d2);
            }
            // if there is an up neighbor, sew sew
            if !last_row {
                let up_neighbor = d1 + (4 * n_square_x) as DartIdentifier;
                map.set_beta::<2>(d3, up_neighbor);
                map.set_beta::<2>(up_neighbor, d3);
            }

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
            if last_column {
                // that last column of 0-cell needs special treatment
                // bc there are no "horizontal" associated dart
                let vertex_id = map.vertex_id(d2);
                map.insert_vertex(
                    vertex_id,
                    origin
                        + Vector2(
                            T::from(x_idx + 1).unwrap() * len_per_x,
                            T::from(y_idx).unwrap() * len_per_y,
                        ),
                );
            }
            if last_row {
                // same as the case on x
                let vertex_id = map.vertex_id(d4);
                map.insert_vertex(
                    vertex_id,
                    origin
                        + Vector2(
                            T::from(x_idx).unwrap() * len_per_x,
                            T::from(y_idx + 1).unwrap() * len_per_y,
                        ),
                );
            }
            if last_row & last_column {
                // need to do the upper right corner
                let vertex_id = map.vertex_id(d3);
                map.insert_vertex(
                    vertex_id,
                    origin
                        + Vector2(
                            T::from(x_idx + 1).unwrap() * len_per_x,
                            T::from(y_idx + 1).unwrap() * len_per_y,
                        ),
                );
            }
        });

    // check the number of built faces
    // this is set as debug only because the operation cost scales with map size
    // this can quickly overshadow the exectime of all previous code
    debug_assert_eq!(map.fetch_faces().identifiers.len(), n_square_x * n_square_y);

    map
}

/// Internal grid-building routine
pub fn build_2d_splitgrid<T: CoordsFloat>(
    origin: Vertex2<T>,
    [n_square_x, n_square_y]: [usize; 2],
    [len_per_x, len_per_y]: [T; 2],
    manager: AttrStorageManager,
) -> CMap2<T> {
    let mut map: CMap2<T> =
        CMap2::new_with_undefined_attributes(6 * n_square_x * n_square_y, manager);

    (0..n_square_y)
        // flatten the loop to expose more parallelism
        // this is not a quantified/benchmarked upgrade, just a seemingly good change
        .flat_map(|y_idx| (0..n_square_x).map(move |x_idx| (y_idx, x_idx)))
        .for_each(|(y_idx, x_idx)| {
            // compute dart IDs of the cell
            let d1 = (1 + 6 * (x_idx + n_square_x * y_idx)) as DartIdentifier;
            let (d2, d3, d4, d5, d6) = (d1 + 1, d1 + 2, d1 + 3, d1 + 4, d1 + 5);
            // are we on the last col / row / both?
            let last_column = x_idx == n_square_x - 1;
            let last_row = y_idx == n_square_y - 1;

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

            // if there is a right neighbor, sew sew
            if !last_column {
                let right_neighbor = d1 + 8;
                map.set_beta::<2>(d5, right_neighbor);
                map.set_beta::<2>(right_neighbor, d5);
            }
            // if there is an up neighbor, sew sew
            if !last_row {
                let up_neighbor = d1 + (6 * n_square_x) as DartIdentifier;
                map.set_beta::<2>(d6, up_neighbor);
                map.set_beta::<2>(up_neighbor, d6);
            }

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
            if last_column {
                // that last column of 0-cell needs special treatment
                // bc there are no "horizontal" associated dart
                let vertex_id = map.vertex_id(d5);
                map.insert_vertex(
                    vertex_id,
                    origin
                        + Vector2(
                            T::from(x_idx + 1).unwrap() * len_per_x,
                            T::from(y_idx).unwrap() * len_per_y,
                        ),
                );
            }
            if last_row {
                // same as the case on x
                let vertex_id = map.vertex_id(d3);
                map.insert_vertex(
                    vertex_id,
                    origin
                        + Vector2(
                            T::from(x_idx).unwrap() * len_per_x,
                            T::from(y_idx + 1).unwrap() * len_per_y,
                        ),
                );
            }
            if last_row & last_column {
                // need to do the upper right corner
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
        });

    // check the number of built faces
    // this is set as debug only because the operation cost scales with map size
    // this can quickly overshadow the exectime of all previous code
    debug_assert_eq!(
        map.fetch_faces().identifiers.len(),
        n_square_x * n_square_y * 2
    );

    map
}
