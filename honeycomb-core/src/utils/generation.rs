//! Utility for sample map generation
//!
//! <div class="warning">
//!
//! **This code is only compiled if the `utils` feature is enabled.**
//!
//! </div>
//!
//! This module contains code used for sample map / mesh generation. This is mostly
//! for testing and benchmarking, but could also be hijacked for very (topologically)
//! simple mesh generation, hence this being kept public.

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, DartIdentifier};

// ------ CONTENT

// --- INNER ROUTINES

/// Internal grid-building routine
fn build2_grid<T: CoordsFloat>(
    [n_square_x, n_square_y]: [usize; 2],
    [len_per_x, len_per_y]: [T; 2],
) -> CMap2<T> {
    let mut map: CMap2<T> = CMap2::new(4 * n_square_x * n_square_y);

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
                    (
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
                        (
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
                        (
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
                        (
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
fn build2_splitgrid<T: CoordsFloat>(
    [n_square_x, n_square_y]: [usize; 2],
    [len_per_x, len_per_y]: [T; 2],
) -> CMap2<T> {
    let mut map: CMap2<T> = CMap2::new(6 * n_square_x * n_square_y);

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
                    (
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
                        (
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
                        (
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
                        (
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

// --- PUBLIC API

/// Builder-level error enum
///
/// This enum is used to describe all non-panic errors that can occur when using a builder
/// structure.
#[derive(Debug)]
pub enum BuilderError<'a> {
    /// The builder is missing one or multiple parameters in order to proceed with the requested
    /// operation.
    MissingParameters(&'a str),
    /// One or multiple of the builder's fields are invalid.
    InvalidParameters(&'a str),
}

/// Builder structure for specialized [`CMap2`].
///
/// The user must specify two out of these three characteristics:
///
/// - `n_cells: [usize; 3]` - The number of cells per axis
/// - `len_per_cell: [T; 3]` - The dimensions of cells per axis
/// - `lens: [T; 3]` -The dimensions of the grid per axis
///
/// This can be done using the provided eponymous methods. The structure can then be used to
/// generate a [`CMap2`] using [`GridBuilder::build2`].
///
/// # Generics
///
/// - `T: CoordsFloat` -- Generic type of the future [`CMap2`] instance.
///
/// # Example
///
/// The following code generates a map that can be visualized by running the example
/// `render_squaremap_parameterized`:
///
/// ```rust
/// # fn main() {
/// use honeycomb_core::{CMap2, utils::GridBuilder};
///
/// let map = GridBuilder::default()
///     .n_cells([15, 5, 0])
///     .len_per_cell_x(1.0_f64)
///     .len_per_cell_y(3.0_f64)
///     .build2();
/// # }
/// ```
///
#[derive(Default, Clone)]
pub struct GridBuilder<T: CoordsFloat> {
    n_cells: Option<[usize; 3]>,
    len_per_cell: Option<[T; 3]>,
    lens: Option<[T; 3]>,
    split_quads: bool,
}

macro_rules! setters {
    ($fld: ident, $fldx: ident, $fldy: ident, $fldz: ident, $zero: expr, $fldty: ty) => {
        /// Set values for all dimensions
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fld(mut self, $fld: [$fldty; 3]) -> Self {
            self.$fld = Some($fld);
            self
        }

        /// Set x-axis value
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fldx(mut self, $fld: $fldty) -> Self {
            if let Some([ptr, _, _]) = &mut self.$fld {
                *ptr = $fld;
            } else {
                self.$fld = Some([$fld, $zero, $zero]);
            }
            self
        }

        /// Set y-axis value
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fldy(mut self, $fld: $fldty) -> Self {
            if let Some([_, ptr, _]) = &mut self.$fld {
                *ptr = $fld;
            } else {
                self.$fld = Some([$zero, $fld, $zero]);
            }
            self
        }

        /// Set z-axis value
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fldz(mut self, $fld: $fldty) -> Self {
            if let Some([_, _, ptr]) = &mut self.$fld {
                *ptr = $fld;
            } else {
                self.$fld = Some([$zero, $zero, $fld]);
            }
            self
        }
    };
}

// editing methods
impl<T: CoordsFloat> GridBuilder<T> {
    // n_cells
    setters!(n_cells, n_cells_x, n_cells_y, n_cells_z, 0, usize);

    // len_per_cell
    setters!(
        len_per_cell,
        len_per_cell_x,
        len_per_cell_y,
        len_per_cell_z,
        T::zero(),
        T
    );

    // lens
    setters!(lens, lens_x, lens_y, lens_z, T::zero(), T);

    /// Indicate whether to split quads of the grid
    #[must_use = "unused builder object, consider removing this method call"]
    pub fn split_quads(mut self, split: bool) -> Self {
        self.split_quads = split;
        self
    }
}

macro_rules! check_parameters {
    ($id: ident, $msg: expr) => {
        if $id.is_sign_negative() | $id.is_zero() {
            return Err(BuilderError::InvalidParameters($msg));
        }
    };
}

// building methods
impl<T: CoordsFloat> GridBuilder<T> {
    #[allow(clippy::missing_errors_doc)]
    /// Consumes the builder and produce a [`CMap2`] object.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(map: CMap2)` -- The method used two of the three parameters to generate a [`CMap2`]
    /// instance
    /// - `Err(BuilderError::MissingParameters)` -- The provided information was not sufficient to
    /// create an instance
    /// - `Err(BuilderError::InvalidParameters)` -- Any of the used length is negative or null
    ///
    /// # Panics
    ///
    /// This method may panic if type casting goes wrong during parameters parsing.
    ///
    /// # Example
    ///
    /// See [`GridBuilder`] example.
    ///
    pub fn build2<'a>(self) -> Result<CMap2<T>, BuilderError<'a>> {
        // preprocess parameters
        let (ns_square, lens_per_cell): ([usize; 2], [T; 2]) = match (
            self.n_cells,
            self.len_per_cell,
            self.lens,
        ) {
            // from # cells and lengths per cell
            (Some([nx, ny, _]), Some([lpx, lpy, _]), lens) => {
                if lens.is_some() {
                    println!("W: All three grid parameters were specified, total lengths will be ignored");
                }
                #[rustfmt::skip]
                check_parameters!(lpx, "Specified length per x cell is either null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "Specified length per y cell is either null or negative");
                ([nx, ny], [lpx, lpy])
            }
            // from # cells and total lengths
            (Some([nx, ny, _]), None, Some([lx, ly, _])) => {
                #[rustfmt::skip]
                check_parameters!(lx, "Specified grid length along x is either null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "Specified grid length along y is either null or negative");
                (
                    [nx, ny],
                    [lx / T::from(nx).unwrap(), ly / T::from(ny).unwrap()],
                )
            }
            // from lengths per cell and total lengths
            (None, Some([lpx, lpy, _]), Some([lx, ly, _])) => {
                #[rustfmt::skip]
                check_parameters!(lpx, "Specified length per x cell is either null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "Specified length per y cell is either null or negative");
                #[rustfmt::skip]
                check_parameters!(lx, "Specified grid length along x is either null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "Specified grid length along y is either null or negative");
                (
                    [
                        (lx / lpx).ceil().to_usize().unwrap(),
                        (ly / lpy).ceil().to_usize().unwrap(),
                    ],
                    [lpx, lpy],
                )
            }
            (_, _, _) => {
                return Err(BuilderError::MissingParameters(
                    "GridBuilder: insufficient building parameters",
                ));
            }
        };

        // build
        Ok(if self.split_quads {
            build2_splitgrid(ns_square, lens_per_cell)
        } else {
            build2_grid(ns_square, lens_per_cell)
        })
    }
}

// predefinite constructs
impl<T: CoordsFloat> GridBuilder<T> {
    /// Generate a predefined [`GridBuilder`] object.
    ///
    /// This object can be used to build a 2-map representing an orthogonal mesh made of
    /// `n_square * n_square` square cells.
    ///
    /// # Arguments
    ///
    /// - `n_square: usize` -- Dimension of the desired mesh.
    ///
    /// ## Generics
    ///
    /// - `const T: CoordsFloat` -- Generic parameter of the returned [`GridBuilder`].
    ///
    /// # Return
    ///
    /// Returns a parameterized [`GridBuilder`] that can be used to generate a [`CMap2`] using the
    /// [`GridBuilder::build2`] method.
    ///
    /// # Example
    ///
    /// ```
    /// use honeycomb_core::{CMap2, utils::GridBuilder};
    ///
    /// let cmap: CMap2<f64> = GridBuilder::unit_squares(2).build2().unwrap();
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
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn unit_squares(n_square: usize) -> Self {
        Self {
            n_cells: Some([n_square; 3]),
            len_per_cell: Some([T::one(); 3]),
            ..Default::default()
        }
    }

    /// Generate a predefined [`GridBuilder`] object.
    ///
    /// This object can be used to build a 2-map representing an orthogonal mesh made of
    /// `n_square * n_square` squares, that are split diagonally for a total of
    /// `n_square * n_square * 2` cells.
    ///
    /// # Arguments
    ///
    /// - `n_square: usize` -- Dimension of the desired mesh.
    ///
    /// ## Generics
    ///
    /// - `const T: CoordsFloat` -- Generic parameter of the returned [`GridBuilder`].
    ///
    /// # Return
    ///
    /// Returns a parameterized [`GridBuilder`] that can be used to generate a [`CMap2`] using the
    /// [`GridBuilder::build2`] method.
    ///
    /// # Example
    ///
    /// ```
    /// use honeycomb_core::{CMap2, utils::GridBuilder};
    ///
    /// let cmap: CMap2<f64> = GridBuilder::split_unit_squares(2).build2().unwrap();
    /// ```
    ///
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn split_unit_squares(n_square: usize) -> Self {
        Self {
            n_cells: Some([n_square; 3]),
            len_per_cell: Some([T::one(); 3]),
            split_quads: true,
            ..Default::default()
        }
    }
}

/// Generate a [`CMap2`] representing a mesh made up of squares.
///
/// <div class="warning">
///
/// **This function is deprecated, please use [`GridBuilder::unit_squares`] instead.**
///
/// </div>
///
/// This function builds and returns a 2-map representing a square mesh
/// made of `n_square * n_square` square cells.
#[deprecated(note = "please use the `GridBuilder::unit_squares` function instead")]
#[must_use = "constructed object is not used, consider removing this function call"]
pub fn square_cmap2<T: CoordsFloat>(n_square: usize) -> CMap2<T> {
    build2_grid([n_square, n_square], [T::one(), T::one()])
}

/// Generate a [`CMap2`] representing a mesh made up of squares split diagonally.
///
/// <div class="warning">
///
/// **This function is deprecated, please use [`GridBuilder::unit_squares`] instead.**
///
/// </div>
///
/// This function builds and returns a 2-map representing a square mesh
/// made of `n_square * n_square * 2` triangle cells.
#[deprecated(note = "please use the `GridBuilder::split_unit_squares` function instead")]
#[must_use = "constructed object is not used, consider removing this function call"]
pub fn splitsquare_cmap2<T: CoordsFloat>(n_square: usize) -> CMap2<T> {
    build2_splitgrid([n_square, n_square], [T::one(), T::one()])
}

// ------ TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_nc_lpc_l() {
        let builder = GridBuilder::default()
            .n_cells([4, 4, 0])
            .len_per_cell([1.0_f64, 1.0_f64, 1.0_f64])
            .lens([4.0_f64, 4.0_f64, 4.0_f64]);
        assert!(builder.clone().build2().is_ok());
        assert!(builder.split_quads(true).build2().is_ok());
    }

    #[test]
    fn build_nc_lpc() {
        let builder = GridBuilder::default()
            .n_cells([4, 4, 0])
            .len_per_cell([1.0_f64, 1.0_f64, 1.0_f64]);
        assert!(builder.clone().build2().is_ok());
        assert!(builder.split_quads(true).build2().is_ok());
    }

    #[test]
    fn build_nc_l() {
        let builder = GridBuilder::default()
            .n_cells_x(4)
            .n_cells_y(4)
            .lens([4.0_f64, 4.0_f64, 4.0_f64]);
        assert!(builder.clone().build2().is_ok());
        assert!(builder.split_quads(true).build2().is_ok());
    }

    #[test]
    fn build_lpc_l() {
        let builder = GridBuilder::default()
            .len_per_cell_x(1.0_f64)
            .len_per_cell_y(1.0_f64)
            .lens_x(4.0)
            .lens_y(4.0);
        assert!(builder.clone().build2().is_ok());
        assert!(builder.split_quads(true).build2().is_ok());
    }

    #[test]
    fn build_incomplete() {
        assert!(GridBuilder::default()
            .len_per_cell([1.0_f64, 1.0_f64, 1.0_f64])
            .build2()
            .is_err());
        assert!(<GridBuilder<f64>>::default()
            .n_cells([4, 4, 0])
            .build2()
            .is_err());
        assert!(GridBuilder::default()
            .lens([4.0_f64, 4.0_f64, 4.0_f64])
            .build2()
            .is_err());
    }

    #[test]
    #[should_panic(expected = "Specified length per y cell is either null or negative")]
    fn build_neg_lpc() {
        let tmp = GridBuilder::default()
            .n_cells([4, 4, 0])
            .len_per_cell([1.0_f64, -1.0_f64, 1.0_f64])
            .build2();
        let _ = tmp.unwrap(); // panic on Err(BuilderError::InvalidParameters)
    }

    #[test]
    #[should_panic(expected = "Specified grid length along x is either null or negative")]
    fn build_null_l() {
        let tmp = GridBuilder::default()
            .n_cells([4, 4, 0])
            .lens([0.0_f64, 4.0_f64, 4.0_f64])
            .build2();
        let _ = tmp.unwrap(); // panic on Err(BuilderError::InvalidParameters)
    }

    #[test]
    #[should_panic(expected = "Specified length per x cell is either null or negative")]
    fn build_neg_lpc_neg_l() {
        // lpc are parsed first so their panic msg should be the one to pop
        // x val is parsed first so ...
        let tmp = GridBuilder::default()
            .len_per_cell([-1.0_f64, -1.0_f64, 1.0_f64])
            .lens([0.0_f64, 4.0_f64, 4.0_f64])
            .build2();
        let _ = tmp.unwrap(); // panic on Err(BuilderError::InvalidParameters)
    }

    #[test]
    fn square_cmap2_correctness() {
        let cmap: CMap2<f64> = GridBuilder::unit_squares(2).build2().unwrap();

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

    #[allow(clippy::too_many_lines)]
    #[test]
    fn splitsquare_cmap2_correctness() {
        let cmap: CMap2<f64> = GridBuilder::split_unit_squares(2).build2().unwrap();

        // hardcoded because using a generic loop & dim would just mean
        // reusing the same pattern as the one used during construction

        // face 1
        assert_eq!(cmap.face_id(1), 1);
        assert_eq!(cmap.face_id(2), 1);
        assert_eq!(cmap.face_id(3), 1);

        let mut face = cmap.i_cell::<2>(1);
        assert_eq!(face.next(), Some(1));
        assert_eq!(face.next(), Some(2));
        assert_eq!(face.next(), Some(3));

        assert_eq!(cmap.beta::<1>(1), 2);
        assert_eq!(cmap.beta::<1>(2), 3);
        assert_eq!(cmap.beta::<1>(3), 1);

        assert_eq!(cmap.beta::<2>(1), 0);
        assert_eq!(cmap.beta::<2>(2), 4);
        assert_eq!(cmap.beta::<2>(3), 0);

        // face 4
        assert_eq!(cmap.face_id(4), 4);
        assert_eq!(cmap.face_id(5), 4);
        assert_eq!(cmap.face_id(6), 4);

        let mut face = cmap.i_cell::<2>(4);
        assert_eq!(face.next(), Some(4));
        assert_eq!(face.next(), Some(5));
        assert_eq!(face.next(), Some(6));

        assert_eq!(cmap.beta::<1>(4), 5);
        assert_eq!(cmap.beta::<1>(5), 6);
        assert_eq!(cmap.beta::<1>(6), 4);

        assert_eq!(cmap.beta::<2>(4), 2);
        assert_eq!(cmap.beta::<2>(5), 9);
        assert_eq!(cmap.beta::<2>(6), 13);

        // face 7
        assert_eq!(cmap.face_id(7), 7);
        assert_eq!(cmap.face_id(8), 7);
        assert_eq!(cmap.face_id(9), 7);

        let mut face = cmap.i_cell::<2>(7);
        assert_eq!(face.next(), Some(7));
        assert_eq!(face.next(), Some(8));
        assert_eq!(face.next(), Some(9));

        assert_eq!(cmap.beta::<1>(7), 8);
        assert_eq!(cmap.beta::<1>(8), 9);
        assert_eq!(cmap.beta::<1>(9), 7);

        assert_eq!(cmap.beta::<2>(7), 0);
        assert_eq!(cmap.beta::<2>(8), 10);
        assert_eq!(cmap.beta::<2>(9), 5);

        // face 10
        assert_eq!(cmap.face_id(10), 10);
        assert_eq!(cmap.face_id(11), 10);
        assert_eq!(cmap.face_id(12), 10);

        let mut face = cmap.i_cell::<2>(10);
        assert_eq!(face.next(), Some(10));
        assert_eq!(face.next(), Some(11));
        assert_eq!(face.next(), Some(12));

        assert_eq!(cmap.beta::<1>(10), 11);
        assert_eq!(cmap.beta::<1>(11), 12);
        assert_eq!(cmap.beta::<1>(12), 10);

        assert_eq!(cmap.beta::<2>(10), 8);
        assert_eq!(cmap.beta::<2>(11), 0);
        assert_eq!(cmap.beta::<2>(12), 19);

        // face 13
        assert_eq!(cmap.face_id(13), 13);
        assert_eq!(cmap.face_id(14), 13);
        assert_eq!(cmap.face_id(15), 13);

        let mut face = cmap.i_cell::<2>(13);
        assert_eq!(face.next(), Some(13));
        assert_eq!(face.next(), Some(14));
        assert_eq!(face.next(), Some(15));

        assert_eq!(cmap.beta::<1>(13), 14);
        assert_eq!(cmap.beta::<1>(14), 15);
        assert_eq!(cmap.beta::<1>(15), 13);

        assert_eq!(cmap.beta::<2>(13), 6);
        assert_eq!(cmap.beta::<2>(14), 16);
        assert_eq!(cmap.beta::<2>(15), 0);

        // face 16
        assert_eq!(cmap.face_id(16), 16);
        assert_eq!(cmap.face_id(17), 16);
        assert_eq!(cmap.face_id(18), 16);

        let mut face = cmap.i_cell::<2>(16);
        assert_eq!(face.next(), Some(16));
        assert_eq!(face.next(), Some(17));
        assert_eq!(face.next(), Some(18));

        assert_eq!(cmap.beta::<1>(16), 17);
        assert_eq!(cmap.beta::<1>(17), 18);
        assert_eq!(cmap.beta::<1>(18), 16);

        assert_eq!(cmap.beta::<2>(16), 14);
        assert_eq!(cmap.beta::<2>(17), 21);
        assert_eq!(cmap.beta::<2>(18), 0);

        // face 19
        assert_eq!(cmap.face_id(19), 19);
        assert_eq!(cmap.face_id(20), 19);
        assert_eq!(cmap.face_id(21), 19);

        let mut face = cmap.i_cell::<2>(19);
        assert_eq!(face.next(), Some(19));
        assert_eq!(face.next(), Some(20));
        assert_eq!(face.next(), Some(21));

        assert_eq!(cmap.beta::<1>(19), 20);
        assert_eq!(cmap.beta::<1>(20), 21);
        assert_eq!(cmap.beta::<1>(21), 19);

        assert_eq!(cmap.beta::<2>(19), 12);
        assert_eq!(cmap.beta::<2>(20), 22);
        assert_eq!(cmap.beta::<2>(21), 17);

        // face 22
        assert_eq!(cmap.face_id(22), 22);
        assert_eq!(cmap.face_id(23), 22);
        assert_eq!(cmap.face_id(24), 22);

        let mut face = cmap.i_cell::<2>(22);
        assert_eq!(face.next(), Some(22));
        assert_eq!(face.next(), Some(23));
        assert_eq!(face.next(), Some(24));

        assert_eq!(cmap.beta::<1>(22), 23);
        assert_eq!(cmap.beta::<1>(23), 24);
        assert_eq!(cmap.beta::<1>(24), 22);

        assert_eq!(cmap.beta::<2>(22), 20);
        assert_eq!(cmap.beta::<2>(23), 0);
        assert_eq!(cmap.beta::<2>(24), 0);
    }
}
