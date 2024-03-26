//! Utility for sample map generation
//!
//! This module contains code used for sample map / mesh generation. This is mostly
//! for testing and benchmarking, but could also be hijacked for very (topologically)
//! simple mesh generation, hence this being kept public.

// ------ IMPORTS

use honeycomb_core::{CMap2, CoordsFloat, DartIdentifier, SewPolicy, UnsewPolicy};

// ------ CONTENT

/// Generate a [CMap2] representing a mesh made up of squares.
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
/// - `const T: CoordsFloat` -- Generic parameter of the returned [CMap2].
///
/// # Return / Panic
///
/// Returns a boundary-less [CMap2] of the specified size. The map contains
/// `4 * n_square * n_square` darts and `(n_square + 1) * (n_square + 1)`
/// vertices.
///
/// # Example
///
/// ```
/// use honeycomb_core::CMap2;
/// use honeycomb_utils::generation::square_cmap2;
///
/// let cmap: CMap2<f64> = square_cmap2(2);
/// ```
///
/// The above code generates the following map:
///
/// ![SQUARECMAP2](../../images/CMap2Square.svg)
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
pub fn square_cmap2<T: CoordsFloat>(n_square: usize) -> CMap2<T> {
    let mut map: CMap2<T> = CMap2::new(4 * n_square.pow(2));

    // first, topology
    (0..n_square).for_each(|y_idx| {
        (0..n_square).for_each(|x_idx| {
            let d1 = (1 + 4 * x_idx + n_square * 4 * y_idx) as DartIdentifier;
            let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
            map.one_link(d1, d2);
            map.one_link(d2, d3);
            map.one_link(d3, d4);
            map.one_link(d4, d1);
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

    // then cells
    (0..n_square + 1).for_each(|y_idx| {
        (0..n_square + 1).for_each(|x_idx| {
            // update the associated 0-cell
            if (y_idx < n_square) & (x_idx < n_square) {
                let base_dart = (1 + 4 * x_idx + n_square * 4 * y_idx) as DartIdentifier;
                let vertex_id = map.vertex_id(base_dart);
                map.insert_vertex(
                    vertex_id,
                    (T::from(x_idx).unwrap(), T::from(y_idx).unwrap()),
                );
                let last_column = x_idx == n_square - 1;
                let last_row = y_idx == n_square - 1;
                if last_column {
                    // that last column of 0-cell needs special treatment
                    // bc there are no "horizontal" associated dart
                    let vertex_id = map.vertex_id(base_dart + 1);
                    map.set_vertex(
                        vertex_id,
                        (T::from(x_idx + 1).unwrap(), T::from(y_idx).unwrap()),
                    )
                    .unwrap();
                }
                if last_row {
                    // same as the case on x
                    let vertex_id = map.vertex_id(base_dart + 3);
                    map.insert_vertex(
                        vertex_id,
                        (T::from(x_idx).unwrap(), T::from(y_idx + 1).unwrap()),
                    );
                }
                if last_row & last_column {
                    // need to do the upper right corner
                    let vertex_id = map.vertex_id(base_dart + 2);
                    map.insert_vertex(
                        vertex_id,
                        (T::from(x_idx + 1).unwrap(), T::from(y_idx + 1).unwrap()),
                    );
                }
            }
        })
    });

    // and then build faces
    assert_eq!(map.fetch_faces().identifiers.len(), n_square.pow(2));

    map
}

/// Generate a [CMap2] representing a mesh made up of squares split diagonally.
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
/// - `const T: CoordsFloat` -- Generic parameter of the returned [CMap2].
///
/// # Return / Panic
///
/// Returns a boundary-less [CMap2] of the specified size. The map contains
/// `6 * n_square * n_square` darts and `(n_square + 1) * (n_square + 1)`
/// vertices.
///
/// # Example
///
/// ```
/// use honeycomb_core::CMap2;
/// use honeycomb_utils::generation::splitsquare_cmap2;
///
/// let cmap: CMap2<f64> = splitsquare_cmap2(2);
/// ```
///
/// The above code generates the following map:
///
/// ![SPLITSQUARECMAP2](../../images/CMap2SplitSquare.svg)
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
pub fn splitsquare_cmap2<T: CoordsFloat>(n_square: usize) -> CMap2<T> {
    let mut map: CMap2<T> = square_cmap2(n_square);

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
    assert_eq!(map.fetch_faces().identifiers.len(), n_square.pow(2) * 2);

    map
}

// ------ TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_cmap2_correctness() {
        let cmap: CMap2<f64> = square_cmap2(2);

        // hardcoded because using a generic loop & dim would just mean
        // reusing the same pattern as the one used during construction

        // face 0
        assert_eq!(cmap.face_id(1), 1);
        assert_eq!(cmap.face_id(2), 1);
        assert_eq!(cmap.face_id(3), 1);
        assert_eq!(cmap.face_id(4), 1);

        let mut face = cmap.i_cell::<2>(1);
        assert_eq!(face.next(), Some(1));
        assert_eq!(face.next(), Some(2));
        assert_eq!(face.next(), Some(3));
        assert_eq!(face.next(), Some(4));
        assert_eq!(face.next(), None);

        assert_eq!(cmap.beta::<1>(1), 2);
        assert_eq!(cmap.beta::<1>(2), 3);
        assert_eq!(cmap.beta::<1>(3), 4);
        assert_eq!(cmap.beta::<1>(4), 1);

        assert_eq!(cmap.beta::<2>(1), 0);
        assert_eq!(cmap.beta::<2>(2), 8);
        assert_eq!(cmap.beta::<2>(3), 9);
        assert_eq!(cmap.beta::<2>(4), 0);

        // face 1
        assert_eq!(cmap.face_id(5), 5);
        assert_eq!(cmap.face_id(6), 5);
        assert_eq!(cmap.face_id(7), 5);
        assert_eq!(cmap.face_id(8), 5);

        let mut face = cmap.i_cell::<2>(5);
        assert_eq!(face.next(), Some(5));
        assert_eq!(face.next(), Some(6));
        assert_eq!(face.next(), Some(7));
        assert_eq!(face.next(), Some(8));
        assert_eq!(face.next(), None);

        assert_eq!(cmap.beta::<1>(5), 6);
        assert_eq!(cmap.beta::<1>(6), 7);
        assert_eq!(cmap.beta::<1>(7), 8);
        assert_eq!(cmap.beta::<1>(8), 5);

        assert_eq!(cmap.beta::<2>(5), 0);
        assert_eq!(cmap.beta::<2>(6), 0);
        assert_eq!(cmap.beta::<2>(7), 13);
        assert_eq!(cmap.beta::<2>(8), 2);

        // face 2
        assert_eq!(cmap.face_id(9), 9);
        assert_eq!(cmap.face_id(10), 9);
        assert_eq!(cmap.face_id(11), 9);
        assert_eq!(cmap.face_id(12), 9);

        let mut face = cmap.i_cell::<2>(9);
        assert_eq!(face.next(), Some(9));
        assert_eq!(face.next(), Some(10));
        assert_eq!(face.next(), Some(11));
        assert_eq!(face.next(), Some(12));
        assert_eq!(face.next(), None);

        assert_eq!(cmap.beta::<1>(9), 10);
        assert_eq!(cmap.beta::<1>(10), 11);
        assert_eq!(cmap.beta::<1>(11), 12);
        assert_eq!(cmap.beta::<1>(12), 9);

        assert_eq!(cmap.beta::<2>(9), 3);
        assert_eq!(cmap.beta::<2>(10), 16);
        assert_eq!(cmap.beta::<2>(11), 0);
        assert_eq!(cmap.beta::<2>(12), 0);

        // face 3
        assert_eq!(cmap.face_id(13), 13);
        assert_eq!(cmap.face_id(14), 13);
        assert_eq!(cmap.face_id(15), 13);
        assert_eq!(cmap.face_id(16), 13);

        let mut face = cmap.i_cell::<2>(13);
        assert_eq!(face.next(), Some(13));
        assert_eq!(face.next(), Some(14));
        assert_eq!(face.next(), Some(15));
        assert_eq!(face.next(), Some(16));
        assert_eq!(face.next(), None);

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
    fn splitsquare_cmap2_correctness() {
        let cmap: CMap2<f64> = splitsquare_cmap2(2);

        // hardcoded because using a generic loop & dim would just mean
        // reusing the same pattern as the one used during construction

        // face 1
        assert_eq!(cmap.face_id(1), 1);
        assert_eq!(cmap.face_id(17), 1);
        assert_eq!(cmap.face_id(4), 1);

        let mut face = cmap.i_cell::<2>(1);
        assert_eq!(face.next(), Some(1));
        assert_eq!(face.next(), Some(17));
        assert_eq!(face.next(), Some(4));

        assert_eq!(cmap.beta::<1>(1), 17);
        assert_eq!(cmap.beta::<1>(17), 4);
        assert_eq!(cmap.beta::<1>(4), 1);

        assert_eq!(cmap.beta::<2>(1), 0);
        assert_eq!(cmap.beta::<2>(17), 18);
        assert_eq!(cmap.beta::<2>(4), 0);

        // face 2
        assert_eq!(cmap.face_id(2), 2);
        assert_eq!(cmap.face_id(3), 2);
        assert_eq!(cmap.face_id(18), 2);

        let mut face = cmap.i_cell::<2>(2);
        assert_eq!(face.next(), Some(2));
        assert_eq!(face.next(), Some(3));
        assert_eq!(face.next(), Some(18));

        assert_eq!(cmap.beta::<1>(2), 3);
        assert_eq!(cmap.beta::<1>(3), 18);
        assert_eq!(cmap.beta::<1>(18), 2);

        assert_eq!(cmap.beta::<2>(2), 8);
        assert_eq!(cmap.beta::<2>(3), 9);
        assert_eq!(cmap.beta::<2>(18), 17);

        // face 5
        assert_eq!(cmap.face_id(5), 5);
        assert_eq!(cmap.face_id(19), 5);
        assert_eq!(cmap.face_id(8), 5);

        let mut face = cmap.i_cell::<2>(5);
        assert_eq!(face.next(), Some(5));
        assert_eq!(face.next(), Some(19));
        assert_eq!(face.next(), Some(8));

        assert_eq!(cmap.beta::<1>(5), 19);
        assert_eq!(cmap.beta::<1>(19), 8);
        assert_eq!(cmap.beta::<1>(8), 5);

        assert_eq!(cmap.beta::<2>(5), 0);
        assert_eq!(cmap.beta::<2>(19), 20);
        assert_eq!(cmap.beta::<2>(8), 2);

        // face 6
        assert_eq!(cmap.face_id(6), 6);
        assert_eq!(cmap.face_id(7), 6);
        assert_eq!(cmap.face_id(20), 6);

        let mut face = cmap.i_cell::<2>(6);
        assert_eq!(face.next(), Some(6));
        assert_eq!(face.next(), Some(7));
        assert_eq!(face.next(), Some(20));

        assert_eq!(cmap.beta::<1>(6), 7);
        assert_eq!(cmap.beta::<1>(7), 20);
        assert_eq!(cmap.beta::<1>(20), 6);

        assert_eq!(cmap.beta::<2>(6), 0);
        assert_eq!(cmap.beta::<2>(7), 13);
        assert_eq!(cmap.beta::<2>(20), 19);

        // face 9
        assert_eq!(cmap.face_id(9), 9);
        assert_eq!(cmap.face_id(21), 9);
        assert_eq!(cmap.face_id(12), 9);

        let mut face = cmap.i_cell::<2>(9);
        assert_eq!(face.next(), Some(9));
        assert_eq!(face.next(), Some(21));
        assert_eq!(face.next(), Some(12));

        assert_eq!(cmap.beta::<1>(9), 21);
        assert_eq!(cmap.beta::<1>(21), 12);
        assert_eq!(cmap.beta::<1>(12), 9);

        assert_eq!(cmap.beta::<2>(9), 3);
        assert_eq!(cmap.beta::<2>(21), 22);
        assert_eq!(cmap.beta::<2>(12), 0);

        // face 10
        assert_eq!(cmap.face_id(10), 10);
        assert_eq!(cmap.face_id(11), 10);
        assert_eq!(cmap.face_id(22), 10);

        let mut face = cmap.i_cell::<2>(10);
        assert_eq!(face.next(), Some(10));
        assert_eq!(face.next(), Some(11));
        assert_eq!(face.next(), Some(22));

        assert_eq!(cmap.beta::<1>(10), 11);
        assert_eq!(cmap.beta::<1>(11), 22);
        assert_eq!(cmap.beta::<1>(22), 10);

        assert_eq!(cmap.beta::<2>(10), 16);
        assert_eq!(cmap.beta::<2>(11), 0);
        assert_eq!(cmap.beta::<2>(22), 21);

        // face 13
        assert_eq!(cmap.face_id(13), 13);
        assert_eq!(cmap.face_id(23), 13);
        assert_eq!(cmap.face_id(16), 13);

        let mut face = cmap.i_cell::<2>(13);
        assert_eq!(face.next(), Some(13));
        assert_eq!(face.next(), Some(23));
        assert_eq!(face.next(), Some(16));

        assert_eq!(cmap.beta::<1>(13), 23);
        assert_eq!(cmap.beta::<1>(23), 16);
        assert_eq!(cmap.beta::<1>(16), 13);

        assert_eq!(cmap.beta::<2>(13), 7);
        assert_eq!(cmap.beta::<2>(23), 24);
        assert_eq!(cmap.beta::<2>(16), 10);

        // face 14
        assert_eq!(cmap.face_id(14), 14);
        assert_eq!(cmap.face_id(15), 15);
        assert_eq!(cmap.face_id(24), 24);

        let mut face = cmap.i_cell::<2>(14);
        assert_eq!(face.next(), Some(14));
        assert_eq!(face.next(), Some(15));
        assert_eq!(face.next(), Some(24));

        assert_eq!(cmap.beta::<1>(14), 15);
        assert_eq!(cmap.beta::<1>(15), 24);
        assert_eq!(cmap.beta::<1>(24), 14);

        assert_eq!(cmap.beta::<2>(14), 0);
        assert_eq!(cmap.beta::<2>(15), 0);
        assert_eq!(cmap.beta::<2>(24), 23);
    }
}
