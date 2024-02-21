use crate::{DartIdentifier, SewPolicy, TwoMap, VertexIdentifier};

/// Generate a [TwoMap] representing a mesh made up of squares.
///
/// This function builds and returns a 2-map representing a square mesh
/// made of `n_square * n_square` square cells.
///
/// # Arguments
///
/// - `n_square: usize` -- Dimension of the returned mesh.
///
/// ## Generics
///
/// - `const N_MARKS: usize` -- Generic parameter of the returned [TwoMap]
///
/// # Return / Panic
///
/// Returns a boundary-less [TwoMap] of the specified size. The map contains
/// `4 * n_square * n_square` darts and `(n_square + 1) * (n_square + 1)`
/// vertices.
///
/// # Example
///
/// ```
/// use honeycomb_core::{TwoMap, generation::square_two_map};
///
/// let cmap: TwoMap<1> = square_two_map(2);
/// ```
///
/// The above code generates the following map:
///
/// ![SQUARETWOMAP](../../images/SquareTwoMap.svg)
///
/// Note that *β<sub>1</sub>* is only represented in one cell but is defined
/// Everywhere following the same pattern. Dart indexing is also consistent
/// with the following rules:
///
/// - inside a cell, the first dart is the one on the bottom, pointing towards
///   the right. Increments (and *β<sub>1</sub>*) follow the trigonometric
///   orientation.
/// - cells are ordered from left to right, from the bottom up. The same rule
///   applies for face IDs.
///
pub fn square_two_map<const N_MARKS: usize>(n_square: usize) -> TwoMap<N_MARKS> {
    let mut map: TwoMap<N_MARKS> = TwoMap::new(4 * n_square.pow(2), (n_square + 1).pow(2));

    // first, topology
    (0..n_square).for_each(|y_idx| {
        (0..n_square).for_each(|x_idx| {
            let d1 = (1 + 4 * x_idx + n_square * 4 * y_idx) as DartIdentifier;
            let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
            map.one_sew(d1, d2, SewPolicy::StretchLeft);
            map.one_sew(d2, d3, SewPolicy::StretchLeft);
            map.one_sew(d3, d4, SewPolicy::StretchLeft);
            map.one_sew(d4, d1, SewPolicy::StretchLeft);
            // if there is a right neighbor, sew sew
            if x_idx != n_square - 1 {
                let right_neighbor = d2 + 6;
                map.two_sew(d2, right_neighbor, SewPolicy::StretchLeft);
            }
            // if there is an up neighbor, sew sew
            if y_idx != n_square - 1 {
                let up_neighbor = d1 + (4 * n_square) as DartIdentifier;
                map.two_sew(d3, up_neighbor, SewPolicy::StretchLeft)
            }
        })
    });

    // then geometry
    (0..n_square + 1).for_each(|y_idx| {
        (0..n_square + 1).for_each(|x_idx| {
            // first position the vertex
            let vertex_id = (y_idx * (n_square + 1) + x_idx) as VertexIdentifier;
            map.set_vertex(vertex_id, [x_idx as f64 * 1.0, y_idx as f64 * 1.0])
                .unwrap();
            // update the associated 0-cell
            if (y_idx < n_square) & (x_idx < n_square) {
                let base_dart = (1 + 4 * x_idx + n_square * 4 * y_idx) as DartIdentifier;
                map.i_cell::<0>(base_dart)
                    .iter()
                    .for_each(|dart_id| map.set_vertexid(*dart_id, vertex_id));
                let last_column = x_idx == n_square - 1;
                let last_row = y_idx == n_square - 1;
                if last_column {
                    // that last column of 0-cell needs special treatment
                    // bc there are no "horizontal" associated dart
                    map.i_cell::<0>(base_dart + 1)
                        .iter()
                        .for_each(|dart_id| map.set_vertexid(*dart_id, vertex_id + 1));
                }
                if last_row {
                    // same as the case on x
                    map.i_cell::<0>(base_dart + 3).iter().for_each(|dart_id| {
                        map.set_vertexid(*dart_id, vertex_id + (n_square + 1) as VertexIdentifier)
                    });
                }
                if last_row & last_column {
                    // need to do the upper right corner
                    map.i_cell::<0>(base_dart + 2).iter().for_each(|dart_id| {
                        map.set_vertexid(*dart_id, vertex_id + (n_square + 2) as VertexIdentifier)
                    });
                }
            }
        })
    });

    // and then build faces
    (0..n_square).for_each(|y_idx| {
        (0..n_square).for_each(|x_idx| {
            let base_dart = (1 + 4 * x_idx + n_square * 4 * y_idx) as DartIdentifier;
            _ = map.build_face(base_dart);
        })
    });

    map
}
