//! Utility for sample map generation
//!
//! This module contains code used for sample map / mesh generation. This is mostly
//! for testing and benchmarking, but could also be hijacked for very (topologically)
//! simple mesh generation, hence this being kept public.

// ------ IMPORTS

use honeycomb_core::{DartIdentifier, SewPolicy, TwoMap, UnsewPolicy, VertexIdentifier};

// ------ CONTENT

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
/// use honeycomb_core::TwoMap;
/// use honeycomb_utils::generation::square_two_map;
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
    assert_eq!(map.build_all_faces(), n_square.pow(2));

    map
}

/// Generate a [TwoMap] representing a mesh made up of squares split diagonally.
///
/// This function builds and returns a 2-map representing a square mesh
/// made of `n_square * n_square * 2` triangle cells.
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
/// `6 * n_square * n_square` darts and `(n_square + 1) * (n_square + 1)`
/// vertices.
///
/// # Example
///
/// ```
/// use honeycomb_core::TwoMap;
/// use honeycomb_utils::generation::splitsquare_two_map;
///
/// let cmap: TwoMap<1> = splitsquare_two_map(2);
/// ```
///
/// The above code generates the following map:
///
/// ![SPLITSQUARETWOMAP](../../images/SplitSquareTwoMap.svg)
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
pub fn splitsquare_two_map<const N_MARKS: usize>(n_square: usize) -> TwoMap<N_MARKS> {
    let mut map: TwoMap<N_MARKS> = square_two_map(n_square);

    (0..n_square.pow(2)).for_each(|square| {
        let d1 = (1 + square * 4) as DartIdentifier;
        let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
        // in a parallel impl, we would create all new darts before-hand
        let dsplit1 = map.add_free_darts(2);
        let dsplit2 = dsplit1 + 1;
        map.two_sew(dsplit1, dsplit2, SewPolicy::StretchLeft);
        map.one_unsew(d1, UnsewPolicy::DoNothing);
        map.one_unsew(d3, UnsewPolicy::DoNothing);
        map.one_sew(d1, dsplit1, SewPolicy::StretchLeft);
        map.one_sew(d3, dsplit2, SewPolicy::StretchLeft);
        map.one_sew(dsplit1, d4, SewPolicy::StretchRight);
        map.one_sew(dsplit2, d2, SewPolicy::StretchRight);
    });

    // rebuild faces
    assert_eq!(map.build_all_faces(), n_square.pow(2) * 2);

    map
}

// ------ TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_two_map_correctness() {
        let cmap: TwoMap<1> = square_two_map(2);

        // hardcoded because using a generic loop & dim would just mean
        // reusing the same pattern as the one used during construction

        // face 0
        assert_eq!(cmap.faceid(1), 0);
        assert_eq!(cmap.faceid(2), 0);
        assert_eq!(cmap.faceid(3), 0);
        assert_eq!(cmap.faceid(4), 0);
        assert_eq!(cmap.face(0).corners.len(), 4);
        assert!(cmap.face(0).corners.contains(&cmap.vertexid(1)));
        assert!(cmap.face(0).corners.contains(&cmap.vertexid(2)));
        assert!(cmap.face(0).corners.contains(&cmap.vertexid(3)));
        assert!(cmap.face(0).corners.contains(&cmap.vertexid(4)));
        assert!(cmap.face(0).closed);

        assert_eq!(cmap.beta::<1>(1), 2);
        assert_eq!(cmap.beta::<1>(2), 3);
        assert_eq!(cmap.beta::<1>(3), 4);
        assert_eq!(cmap.beta::<1>(4), 1);

        assert_eq!(cmap.beta::<2>(1), 0);
        assert_eq!(cmap.beta::<2>(2), 8);
        assert_eq!(cmap.beta::<2>(3), 9);
        assert_eq!(cmap.beta::<2>(4), 0);

        // face 1
        assert_eq!(cmap.faceid(5), 1);
        assert_eq!(cmap.faceid(6), 1);
        assert_eq!(cmap.faceid(7), 1);
        assert_eq!(cmap.faceid(8), 1);
        assert_eq!(cmap.face(1).corners.len(), 4);
        assert!(cmap.face(1).corners.contains(&cmap.vertexid(5)));
        assert!(cmap.face(1).corners.contains(&cmap.vertexid(6)));
        assert!(cmap.face(1).corners.contains(&cmap.vertexid(7)));
        assert!(cmap.face(1).corners.contains(&cmap.vertexid(8)));
        assert!(cmap.face(1).closed);

        assert_eq!(cmap.beta::<1>(5), 6);
        assert_eq!(cmap.beta::<1>(6), 7);
        assert_eq!(cmap.beta::<1>(7), 8);
        assert_eq!(cmap.beta::<1>(8), 5);

        assert_eq!(cmap.beta::<2>(5), 0);
        assert_eq!(cmap.beta::<2>(6), 0);
        assert_eq!(cmap.beta::<2>(7), 13);
        assert_eq!(cmap.beta::<2>(8), 2);

        // face 2
        assert_eq!(cmap.faceid(9), 2);
        assert_eq!(cmap.faceid(10), 2);
        assert_eq!(cmap.faceid(11), 2);
        assert_eq!(cmap.faceid(12), 2);
        assert_eq!(cmap.face(2).corners.len(), 4);
        assert!(cmap.face(2).corners.contains(&cmap.vertexid(9)));
        assert!(cmap.face(2).corners.contains(&cmap.vertexid(10)));
        assert!(cmap.face(2).corners.contains(&cmap.vertexid(11)));
        assert!(cmap.face(2).corners.contains(&cmap.vertexid(12)));
        assert!(cmap.face(2).closed);

        assert_eq!(cmap.beta::<1>(9), 10);
        assert_eq!(cmap.beta::<1>(10), 11);
        assert_eq!(cmap.beta::<1>(11), 12);
        assert_eq!(cmap.beta::<1>(12), 9);

        assert_eq!(cmap.beta::<2>(9), 3);
        assert_eq!(cmap.beta::<2>(10), 16);
        assert_eq!(cmap.beta::<2>(11), 0);
        assert_eq!(cmap.beta::<2>(12), 0);

        // face 3
        assert_eq!(cmap.faceid(13), 3);
        assert_eq!(cmap.faceid(14), 3);
        assert_eq!(cmap.faceid(15), 3);
        assert_eq!(cmap.faceid(16), 3);
        assert_eq!(cmap.face(3).corners.len(), 4);
        assert!(cmap.face(3).corners.contains(&cmap.vertexid(13)));
        assert!(cmap.face(3).corners.contains(&cmap.vertexid(14)));
        assert!(cmap.face(3).corners.contains(&cmap.vertexid(15)));
        assert!(cmap.face(3).corners.contains(&cmap.vertexid(16)));
        assert!(cmap.face(3).closed);

        assert_eq!(cmap.beta::<1>(13), 14);
        assert_eq!(cmap.beta::<1>(14), 15);
        assert_eq!(cmap.beta::<1>(15), 16);
        assert_eq!(cmap.beta::<1>(16), 13);

        assert_eq!(cmap.beta::<2>(13), 7);
        assert_eq!(cmap.beta::<2>(14), 0);
        assert_eq!(cmap.beta::<2>(15), 0);
        assert_eq!(cmap.beta::<2>(16), 10);
    }

    #[test]
    fn splitsquare_two_map_correctness() {
        let cmap: TwoMap<1> = splitsquare_two_map(2);

        // hardcoded because using a generic loop & dim would just mean
        // reusing the same pattern as the one used during construction

        // face 0
        assert_eq!(cmap.faceid(1), 0);
        assert_eq!(cmap.faceid(17), 0);
        assert_eq!(cmap.faceid(4), 0);
        assert_eq!(cmap.face(0).corners.len(), 3);
        assert!(cmap.face(0).corners.contains(&cmap.vertexid(1)));
        assert!(cmap.face(0).corners.contains(&cmap.vertexid(17)));
        assert!(cmap.face(0).corners.contains(&cmap.vertexid(4)));
        assert!(cmap.face(0).closed);

        assert_eq!(cmap.beta::<1>(1), 17);
        assert_eq!(cmap.beta::<1>(17), 4);
        assert_eq!(cmap.beta::<1>(4), 1);

        assert_eq!(cmap.beta::<2>(1), 0);
        assert_eq!(cmap.beta::<2>(17), 18);
        assert_eq!(cmap.beta::<2>(4), 0);

        // face 1
        assert_eq!(cmap.faceid(2), 1);
        assert_eq!(cmap.faceid(3), 1);
        assert_eq!(cmap.faceid(18), 1);
        assert_eq!(cmap.face(1).corners.len(), 3);
        assert!(cmap.face(1).corners.contains(&cmap.vertexid(2)));
        assert!(cmap.face(1).corners.contains(&cmap.vertexid(3)));
        assert!(cmap.face(1).corners.contains(&cmap.vertexid(18)));
        assert!(cmap.face(1).closed);

        assert_eq!(cmap.beta::<1>(2), 3);
        assert_eq!(cmap.beta::<1>(3), 18);
        assert_eq!(cmap.beta::<1>(18), 2);

        assert_eq!(cmap.beta::<2>(2), 8);
        assert_eq!(cmap.beta::<2>(3), 9);
        assert_eq!(cmap.beta::<2>(18), 17);

        // face 2
        assert_eq!(cmap.faceid(5), 2);
        assert_eq!(cmap.faceid(19), 2);
        assert_eq!(cmap.faceid(8), 2);
        assert_eq!(cmap.face(2).corners.len(), 3);
        assert!(cmap.face(2).corners.contains(&cmap.vertexid(5)));
        assert!(cmap.face(2).corners.contains(&cmap.vertexid(19)));
        assert!(cmap.face(2).corners.contains(&cmap.vertexid(8)));
        assert!(cmap.face(2).closed);

        assert_eq!(cmap.beta::<1>(5), 19);
        assert_eq!(cmap.beta::<1>(19), 8);
        assert_eq!(cmap.beta::<1>(8), 5);

        assert_eq!(cmap.beta::<2>(5), 0);
        assert_eq!(cmap.beta::<2>(19), 20);
        assert_eq!(cmap.beta::<2>(8), 2);

        // face 3
        assert_eq!(cmap.faceid(6), 3);
        assert_eq!(cmap.faceid(7), 3);
        assert_eq!(cmap.faceid(20), 3);
        assert_eq!(cmap.face(3).corners.len(), 3);
        assert!(cmap.face(3).corners.contains(&cmap.vertexid(6)));
        assert!(cmap.face(3).corners.contains(&cmap.vertexid(7)));
        assert!(cmap.face(3).corners.contains(&cmap.vertexid(20)));
        assert!(cmap.face(3).closed);

        assert_eq!(cmap.beta::<1>(6), 7);
        assert_eq!(cmap.beta::<1>(7), 20);
        assert_eq!(cmap.beta::<1>(20), 6);

        assert_eq!(cmap.beta::<2>(6), 0);
        assert_eq!(cmap.beta::<2>(7), 13);
        assert_eq!(cmap.beta::<2>(20), 19);

        // face 4
        assert_eq!(cmap.faceid(9), 4);
        assert_eq!(cmap.faceid(21), 4);
        assert_eq!(cmap.faceid(12), 4);
        assert_eq!(cmap.face(4).corners.len(), 3);
        assert!(cmap.face(4).corners.contains(&cmap.vertexid(9)));
        assert!(cmap.face(4).corners.contains(&cmap.vertexid(21)));
        assert!(cmap.face(4).corners.contains(&cmap.vertexid(12)));
        assert!(cmap.face(4).closed);

        assert_eq!(cmap.beta::<1>(9), 21);
        assert_eq!(cmap.beta::<1>(21), 12);
        assert_eq!(cmap.beta::<1>(12), 9);

        assert_eq!(cmap.beta::<2>(9), 3);
        assert_eq!(cmap.beta::<2>(21), 22);
        assert_eq!(cmap.beta::<2>(12), 0);

        // face 5
        assert_eq!(cmap.faceid(10), 5);
        assert_eq!(cmap.faceid(11), 5);
        assert_eq!(cmap.faceid(22), 5);
        assert_eq!(cmap.face(5).corners.len(), 3);
        assert!(cmap.face(5).corners.contains(&cmap.vertexid(10)));
        assert!(cmap.face(5).corners.contains(&cmap.vertexid(11)));
        assert!(cmap.face(5).corners.contains(&cmap.vertexid(22)));
        assert!(cmap.face(5).closed);

        assert_eq!(cmap.beta::<1>(10), 11);
        assert_eq!(cmap.beta::<1>(11), 22);
        assert_eq!(cmap.beta::<1>(22), 10);

        assert_eq!(cmap.beta::<2>(10), 16);
        assert_eq!(cmap.beta::<2>(11), 0);
        assert_eq!(cmap.beta::<2>(22), 21);

        // face 6
        assert_eq!(cmap.faceid(13), 6);
        assert_eq!(cmap.faceid(23), 6);
        assert_eq!(cmap.faceid(16), 6);
        assert_eq!(cmap.face(6).corners.len(), 3);
        assert!(cmap.face(6).corners.contains(&cmap.vertexid(13)));
        assert!(cmap.face(6).corners.contains(&cmap.vertexid(23)));
        assert!(cmap.face(6).corners.contains(&cmap.vertexid(16)));
        assert!(cmap.face(6).closed);

        assert_eq!(cmap.beta::<1>(13), 23);
        assert_eq!(cmap.beta::<1>(23), 16);
        assert_eq!(cmap.beta::<1>(16), 13);

        assert_eq!(cmap.beta::<2>(13), 7);
        assert_eq!(cmap.beta::<2>(23), 24);
        assert_eq!(cmap.beta::<2>(16), 10);

        // face 7
        assert_eq!(cmap.faceid(14), 7);
        assert_eq!(cmap.faceid(15), 7);
        assert_eq!(cmap.faceid(24), 7);
        assert_eq!(cmap.face(7).corners.len(), 3);
        assert!(cmap.face(7).corners.contains(&cmap.vertexid(14)));
        assert!(cmap.face(7).corners.contains(&cmap.vertexid(15)));
        assert!(cmap.face(7).corners.contains(&cmap.vertexid(24)));
        assert!(cmap.face(7).closed);

        assert_eq!(cmap.beta::<1>(14), 15);
        assert_eq!(cmap.beta::<1>(15), 24);
        assert_eq!(cmap.beta::<1>(24), 14);

        assert_eq!(cmap.beta::<2>(14), 0);
        assert_eq!(cmap.beta::<2>(15), 0);
        assert_eq!(cmap.beta::<2>(24), 23);
    }
}
