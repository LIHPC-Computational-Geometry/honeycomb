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

    // first, topology
    (0..n_square_y).for_each(|y_idx| {
        (0..n_square_x).for_each(|x_idx| {
            let d1 = (1 + 4 * x_idx + n_square_x * 4 * y_idx) as DartIdentifier;
            let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
            map.one_link(d1, d2);
            map.one_link(d2, d3);
            map.one_link(d3, d4);
            map.one_link(d4, d1);
            // if there is a right neighbor, sew sew
            if x_idx != n_square_x - 1 {
                let right_neighbor = d2 + 6;
                map.two_link(d2, right_neighbor);
            }
            // if there is an up neighbor, sew sew
            if y_idx != n_square_y - 1 {
                let up_neighbor = d1 + (4 * n_square_x) as DartIdentifier;
                map.two_link(d3, up_neighbor);
            }
        });
    });

    // then cells
    (0..=n_square_y).for_each(|y_idx| {
        (0..=n_square_x).for_each(|x_idx| {
            // update the associated 0-cell
            if (y_idx < n_square_y) & (x_idx < n_square_x) {
                let base_dart = (1 + 4 * x_idx + n_square_x * 4 * y_idx) as DartIdentifier;
                let vertex_id = map.vertex_id(base_dart);
                map.insert_vertex(
                    vertex_id,
                    origin
                        + Vector2(
                            T::from(x_idx).unwrap() * len_per_x,
                            T::from(y_idx).unwrap() * len_per_y,
                        ),
                );
                let last_column = x_idx == n_square_x - 1;
                let last_row = y_idx == n_square_y - 1;
                if last_column {
                    // that last column of 0-cell needs special treatment
                    // bc there are no "horizontal" associated dart
                    let vertex_id = map.vertex_id(base_dart + 1);
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
                    let vertex_id = map.vertex_id(base_dart + 3);
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
                    let vertex_id = map.vertex_id(base_dart + 2);
                    map.insert_vertex(
                        vertex_id,
                        origin
                            + Vector2(
                                T::from(x_idx + 1).unwrap() * len_per_x,
                                T::from(y_idx + 1).unwrap() * len_per_y,
                            ),
                    );
                }
            }
        });
    });

    // and then build faces
    assert_eq!(map.fetch_faces().identifiers.len(), n_square_x * n_square_y);

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

    // first, topology
    (0..n_square_y).for_each(|y_idx| {
        (0..n_square_x).for_each(|x_idx| {
            let d1 = (1 + 6 * (x_idx + n_square_x * y_idx)) as DartIdentifier;
            let (d2, d3, d4, d5, d6) = (d1 + 1, d1 + 2, d1 + 3, d1 + 4, d1 + 5);
            // bottom left triangle
            map.one_link(d1, d2);
            map.one_link(d2, d3);
            map.one_link(d3, d1);
            // top right triangle
            map.one_link(d4, d5);
            map.one_link(d5, d6);
            map.one_link(d6, d4);
            // diagonal
            map.two_link(d2, d4);

            // if there is a right neighbor, sew sew
            if x_idx != n_square_x - 1 {
                let right_neighbor = d1 + 8;
                map.two_link(d5, right_neighbor);
            }
            // if there is an up neighbor, sew sew
            if y_idx != n_square_x - 1 {
                let up_neighbor = d1 + (6 * n_square_x) as DartIdentifier;
                map.two_link(d6, up_neighbor);
            }
        });
    });

    // then cells
    (0..=n_square_y).for_each(|y_idx| {
        (0..=n_square_x).for_each(|x_idx| {
            // update the associated 0-cell
            if (y_idx < n_square_y) & (x_idx < n_square_x) {
                let base_dart = (1 + 6 * (x_idx + n_square_x * y_idx)) as DartIdentifier;
                let vertex_id = map.vertex_id(base_dart);
                map.insert_vertex(
                    vertex_id,
                    origin
                        + Vector2(
                            T::from(x_idx).unwrap() * len_per_x,
                            T::from(y_idx).unwrap() * len_per_y,
                        ),
                );
                let last_column = x_idx == n_square_x - 1;
                let last_row = y_idx == n_square_y - 1;
                if last_column {
                    // that last column of 0-cell needs special treatment
                    // bc there are no "horizontal" associated dart
                    let vertex_id = map.vertex_id(base_dart + 4);
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
                    let vertex_id = map.vertex_id(base_dart + 2);
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
                    let vertex_id = map.vertex_id(base_dart + 5);
                    map.insert_vertex(
                        vertex_id,
                        origin
                            + Vector2(
                                T::from(x_idx + 1).unwrap() * len_per_x,
                                T::from(y_idx + 1).unwrap() * len_per_y,
                            ),
                    );
                }
            }
        });
    });
    // rebuild faces
    assert_eq!(
        map.fetch_faces().identifiers.len(),
        n_square_x * n_square_y * 2
    );

    map
}
