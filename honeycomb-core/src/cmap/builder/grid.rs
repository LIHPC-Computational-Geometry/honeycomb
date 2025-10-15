use crate::attributes::AttrStorageManager;
use crate::cmap::{BuilderError, CMap2, CMap3, DartIdType, VertexIdType};
use crate::geometry::{CoordsFloat, Vector2, Vector3, Vertex2, Vertex3};

// --- grid descriptor

/// # Grid description used by the map builder
///
/// The user must specify two out of these three characteristics (third is deduced):
///
/// - `n_cells: [usize; D]` -- The number of cells per axis
/// - `len_per_cell: [T; D]` -- The dimensions of cells per axis
/// - `lens: [T; D]` -- The total dimensions of the grid per axis
///
/// ## Generics
///
/// - `const D: usize` -- Dimension of the grid. Should be 2 or 3.
/// - `T: CoordsFloat` -- Generic FP type that will be used by the map's vertices.
#[derive(Clone)]
pub struct GridDescriptor<const D: usize, T: CoordsFloat> {
    pub(crate) origin: [T; D],
    pub(crate) n_cells: Option<[usize; D]>,
    pub(crate) len_per_cell: Option<[T; D]>,
    pub(crate) lens: Option<[T; D]>,
    pub(crate) split_cells: bool,
}

impl<const D: usize, T: CoordsFloat> Default for GridDescriptor<D, T> {
    fn default() -> Self {
        Self {
            origin: [T::zero(); D],
            n_cells: None,
            len_per_cell: None,
            lens: None,
            split_cells: false,
        }
    }
}

impl<const D: usize, T: CoordsFloat> GridDescriptor<D, T> {
    /// Set values for all dimensions
    #[must_use = "unused builder object"]
    pub fn n_cells(mut self, n_cells: [usize; D]) -> Self {
        self.n_cells = Some(n_cells);
        self
    }

    /// Set values for all dimensions
    #[must_use = "unused builder object"]
    pub fn len_per_cell(mut self, len_per_cell: [T; D]) -> Self {
        self.len_per_cell = Some(len_per_cell);
        self
    }

    /// Set values for all dimensions
    #[must_use = "unused builder object"]
    pub fn lens(mut self, lens: [T; D]) -> Self {
        self.lens = Some(lens);
        self
    }

    /// Set origin (most bottom-left vertex) of the grid
    #[must_use = "unused builder object"]
    pub fn origin(mut self, origin: [T; D]) -> Self {
        self.origin = origin;
        self
    }

    /// Indicate whether to split cells of the grid
    ///
    /// In 2D, this will result in triangular cells.
    ///
    /// In 3D, this will result in tetrahedral cells.
    #[must_use = "unused builder object"]
    pub fn split_cells(mut self, split: bool) -> Self {
        self.split_cells = split;
        self
    }
}

// --- parsing routine

macro_rules! check_parameters {
    ($id: ident, $msg: expr) => {
        if $id.is_sign_negative() | $id.is_zero() {
            return Err(BuilderError::InvalidGridParameters($msg));
        }
    };
}

impl<T: CoordsFloat> GridDescriptor<2, T> {
    /// Parse provided grid parameters to provide what's used to actually generate the grid.
    #[allow(clippy::type_complexity)]
    pub(crate) fn parse_2d(self) -> Result<(Vertex2<T>, [usize; 2], [T; 2]), BuilderError> {
        match (self.n_cells, self.len_per_cell, self.lens) {
            // from # cells and lengths per cell
            (Some([nx, ny]), Some([lpx, lpy]), lens) => {
                if lens.is_some() {
                    eprintln!(
                        "W: All three grid parameters were specified, total lengths will be ignored"
                    );
                }
                #[rustfmt::skip]
                check_parameters!(lpx, "length per x cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "length per y cell is null or negative");
                Ok((
                    Vertex2(self.origin[0], self.origin[1]),
                    [nx, ny],
                    [lpx, lpy],
                ))
            }
            // from # cells and total lengths
            (Some([nx, ny]), None, Some([lx, ly])) => {
                #[rustfmt::skip]
                check_parameters!(lx, "grid length along x is null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "grid length along y is null or negative");
                Ok((
                    Vertex2(self.origin[0], self.origin[1]),
                    [nx, ny],
                    [lx / T::from(nx).unwrap(), ly / T::from(ny).unwrap()],
                ))
            }
            // from lengths per cell and total lengths
            (None, Some([lpx, lpy]), Some([lx, ly])) => {
                #[rustfmt::skip]
                check_parameters!(lpx, "length per x cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "length per y cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lx, "grid length along x is null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "grid length along y is null or negative");
                Ok((
                    Vertex2(self.origin[0], self.origin[1]),
                    [
                        (lx / lpx).ceil().to_usize().unwrap(),
                        (ly / lpy).ceil().to_usize().unwrap(),
                    ],
                    [lpx, lpy],
                ))
            }
            (_, _, _) => Err(BuilderError::MissingGridParameters),
        }
    }
}

impl<T: CoordsFloat> GridDescriptor<3, T> {
    /// Parse provided grid parameters to provide what's used to actually generate the grid.
    #[allow(clippy::type_complexity)]
    pub(crate) fn parse_3d(self) -> Result<(Vertex3<T>, [usize; 3], [T; 3]), BuilderError> {
        match (self.n_cells, self.len_per_cell, self.lens) {
            // from # cells and lengths per cell
            (Some([nx, ny, nz]), Some([lpx, lpy, lpz]), lens) => {
                if lens.is_some() {
                    eprintln!(
                        "W: All three grid parameters were specified, total lengths will be ignored"
                    );
                }
                #[rustfmt::skip]
                check_parameters!(lpx, "length per x cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "length per y cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpz, "length per z cell is null or negative");
                Ok((
                    Vertex3(self.origin[0], self.origin[1], self.origin[2]),
                    [nx, ny, nz],
                    [lpx, lpy, lpz],
                ))
            }
            // from # cells and total lengths
            (Some([nx, ny, nz]), None, Some([lx, ly, lz])) => {
                #[rustfmt::skip]
                check_parameters!(lx, "grid length along x is null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "grid length along y is null or negative");
                #[rustfmt::skip]
                check_parameters!(lz, "grid length along z is null or negative");
                Ok((
                    Vertex3(self.origin[0], self.origin[1], self.origin[2]),
                    [nx, ny, nz],
                    [
                        lx / T::from(nx).unwrap(),
                        ly / T::from(ny).unwrap(),
                        lz / T::from(nz).unwrap(),
                    ],
                ))
            }
            // from lengths per cell and total lengths
            (None, Some([lpx, lpy, lpz]), Some([lx, ly, lz])) => {
                #[rustfmt::skip]
                check_parameters!(lpx, "length per x cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "length per y cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lpz, "length per z cell is null or negative");
                #[rustfmt::skip]
                check_parameters!(lx, "grid length along x is null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "grid length along y is null or negative");
                #[rustfmt::skip]
                check_parameters!(lz, "grid length along z is null or negative");
                Ok((
                    Vertex3(self.origin[0], self.origin[1], self.origin[2]),
                    [
                        (lx / lpx).ceil().to_usize().unwrap(),
                        (ly / lpy).ceil().to_usize().unwrap(),
                        (lz / lpz).ceil().to_usize().unwrap(),
                    ],
                    [lpx, lpy, lpz],
                ))
            }
            (_, _, _) => Err(BuilderError::MissingGridParameters),
        }
    }
}

// --- building routines

// ------ 2D

/// Internal grid-building routine
#[allow(clippy::too_many_lines)]
pub fn build_2d_grid<T: CoordsFloat>(
    origin: Vertex2<T>,
    [n_square_x, n_square_y]: [usize; 2],
    [len_per_x, len_per_y]: [T; 2],
    manager: AttrStorageManager,
) -> CMap2<T> {
    let map: CMap2<T> = CMap2::new_with_undefined_attributes(4 * n_square_x * n_square_y, manager);

    // init beta functions
    (1..=(4 * n_square_x * n_square_y) as DartIdType)
        .zip(generate_square_beta_values(n_square_x, n_square_y))
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
    debug_assert_eq!(map.iter_faces().count(), n_square_x * n_square_y);

    map
}

#[allow(clippy::inline_always)]
#[rustfmt::skip]
#[inline(always)]
fn generate_square_beta_values(n_x: usize, n_y: usize) -> impl Iterator<Item = [DartIdType; 3]> {
    // this loop hierarchy yields the value in correct order
    // left to right first, then bottom to top
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

    // init beta functions
    (1..=(6 * n_square_x * n_square_y) as DartIdType)
        .zip(generate_tris_beta_values(n_square_x, n_square_y))
        .for_each(|(dart, images)| {
            map.set_betas(dart, images);
        });

    // place vertices

    // bottow left vertex of all cells
    (0..n_square_y)
        // flatten the loop to expose more parallelism
        .flat_map(|y_idx| (0..n_square_x).map(move |x_idx| (y_idx, x_idx)))
        .for_each(|(y_idx, x_idx)| {
            let vertex_id = map.vertex_id((1 + x_idx * 6 + y_idx * 6 * n_square_x) as DartIdType);
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
        let vertex_id = map.vertex_id((4 + x_idx * 6 + y_idx * 6 * n_square_x) as DartIdType);
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
        let vertex_id = map.vertex_id((2 + x_idx * 6 + y_idx * 6 * n_square_x) as DartIdType);
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
        let vertex_id = map.vertex_id((6 + x_idx * 6 + y_idx * 6 * n_square_x) as DartIdType); // top right
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
    debug_assert_eq!(map.iter_faces().count(), 2 * n_square_x * n_square_y);

    map
}

#[allow(clippy::inline_always)]
#[rustfmt::skip]
#[inline(always)]
fn generate_tris_beta_values(n_x: usize, n_y: usize) -> impl Iterator<Item = [DartIdType; 3]> {
    // this loop hierarchy yields the value in correct order
    // left to right first, then bottom to top
    (0..n_y).flat_map(move |iy| {
        (0..n_x).flat_map(move |ix| {
                let d1 = (1 + 6 * ix + n_x * 6 * iy) as DartIdType;
                let (d2, d3, d4, d5, d6) = (d1 + 1, d1 + 2, d1 + 3, d1 + 4, d1 + 5);
                // beta images of [d1, d2, d3, d4]
                [
                    [ d3, d2, if iy == 0     { 0 } else { d6 - 6 * n_x as DartIdType } ],
                    [ d1, d3, d4                                                       ],
                    [ d2, d1, if ix == 0     { 0 } else { d5 - 6                     } ],
                    [ d6, d5, d2                                                       ],
                    [ d4, d6, if ix == n_x-1 { 0 } else { d3 + 6                     } ],
                    [ d5, d4, if iy == n_y-1 { 0 } else { d1 + 6 * n_x as DartIdType } ],
                ]
                .into_iter()
            })
        })
}

// ------ 3D

/// Internal grid-building routine
#[allow(clippy::too_many_lines)]
pub fn build_3d_grid<T: CoordsFloat>(
    origin: Vertex3<T>,
    n_cells_per_axis: [usize; 3],
    lengths: [T; 3],
    manager: AttrStorageManager,
) -> CMap3<T> {
    let [n_square_x, n_square_y, n_square_z] = n_cells_per_axis;
    let n_darts = 24 * n_square_x * n_square_y * n_square_z;

    let map: CMap3<T> = CMap3::new_with_undefined_attributes(n_darts, manager);

    // init beta functions
    (1..=n_darts as DartIdType)
        .zip(generate_hex_beta_values(n_cells_per_axis))
        .for_each(|(dart, images)| {
            map.set_betas(dart, images);
        });

    // place vertices
    (1..=n_darts as DartIdType)
        .filter(|d| *d as VertexIdType == map.vertex_id(*d))
        .for_each(|d| {
            let v = origin + generate_hex_offset(d, n_cells_per_axis, lengths);
            map.force_write_vertex(d as VertexIdType, v);
        });

    // check the number of built volumes
    // this is set as debug only because the operation cost scales with map size
    // this can quickly overshadow the exectime of all previous code
    debug_assert_eq!(
        map.iter_volumes().count(),
        n_square_x * n_square_y * n_square_z
    );

    map
}

//
// y+
// |
// |
// |
// +------x+
//  \
//   \
//    z+
//
// faces:
// y-: 1
// y+: 21
// z-: 5
// z+: 13
// x-: 17
// x+: 9
#[allow(clippy::inline_always)]
#[rustfmt::skip]
#[inline(always)]
fn generate_hex_beta_values(
    [n_x, n_y, n_z]: [usize; 3],
) -> impl Iterator<Item = [DartIdType; 4]> {
    // this loop hierarchy yields the value in correct order
    // left to right first, then bottom to top
    (0..n_z).flat_map(move |iz| {
        (0..n_y).flat_map(move |iy| {
            (0..n_x).flat_map(move |ix| {
                let d1 = (1 + 24 * ix + n_x * 24 * iy + n_x * n_y * 24 * iz) as DartIdType;
                let (    d2 , d3 , d4 , d5 , d6 , d7 , d8 ,
                    d9 , d10, d11, d12, d13, d14, d15, d16,
                    d17, d18, d19, d20, d21, d22, d23, d24,
                ) = (        d1 + 1 , d1 + 2 , d1 + 3 , d1 + 4 , d1 + 5 , d1 + 6 , d1 + 7 ,
                    d1 + 8 , d1 + 9 , d1 + 10, d1 + 11, d1 + 12, d1 + 13, d1 + 14, d1 + 15,
                    d1 + 16, d1 + 17, d1 + 18, d1 + 19, d1 + 20, d1 + 21, d1 + 22, d1 + 23,
                );
                let noffset_x = 24;
                let noffset_y = noffset_x * n_x as DartIdType;
                let noffset_z = noffset_y * n_y  as DartIdType;

                // beta images of the cube (tm)
                [
                    // down (1, y-)
                    [d4 , d2 , d5 , if iy == 0       { 0 } else { d21 - noffset_y }],
                    [d1 , d3 , d9 , if iy == 0       { 0 } else { d24 - noffset_y }],
                    [d2 , d4 , d13, if iy == 0       { 0 } else { d23 - noffset_y }],
                    [d3 , d1 , d17, if iy == 0       { 0 } else { d22 - noffset_y }],
                    // side (5 , z-)
                    [d8 , d6 , d1 , if iz == 0       { 0 } else { d13 - noffset_z }],
                    [d5 , d7 , d20, if iz == 0       { 0 } else { d16 - noffset_z }],
                    [d6 , d8 , d21, if iz == 0       { 0 } else { d15 - noffset_z }],
                    [d7 , d5 , d10, if iz == 0       { 0 } else { d14 - noffset_z }],
                    // side (9 , x+)
                    [d12, d10, d2 , if ix == n_x - 1 { 0 } else { d17 + noffset_x }],
                    [d9 , d11, d8 , if ix == n_x - 1 { 0 } else { d20 + noffset_x }],
                    [d10, d12, d24, if ix == n_x - 1 { 0 } else { d19 + noffset_x }],
                    [d11, d9 , d14, if ix == n_x - 1 { 0 } else { d18 + noffset_x }],
                    // side (13, z+)
                    [d16, d14, d3 , if iz == n_z - 1 { 0 } else { d5  + noffset_z }],
                    [d13, d15, d12, if iz == n_z - 1 { 0 } else { d8  + noffset_z }],
                    [d14, d16, d23, if iz == n_z - 1 { 0 } else { d7  + noffset_z }],
                    [d15, d13, d18, if iz == n_z - 1 { 0 } else { d6  + noffset_z }],
                    // side (17, x-)
                    [d20, d18, d4 , if ix == 0       { 0 } else { d9  - noffset_x }],
                    [d17, d19, d16, if ix == 0       { 0 } else { d12 - noffset_x }],
                    [d18, d20, d22, if ix == 0       { 0 } else { d11 - noffset_x }],
                    [d19, d17, d6 , if ix == 0       { 0 } else { d10 - noffset_x }],
                    // up   (21, y+)
                    [d24, d22, d7 , if iy == n_y - 1 { 0 } else { d1  + noffset_y }],
                    [d21, d23, d19, if iy == n_y - 1 { 0 } else { d4  + noffset_y }],
                    [d22, d24, d15, if iy == n_y - 1 { 0 } else { d3  + noffset_y }],
                    [d23, d21, d11, if iy == n_y - 1 { 0 } else { d2  + noffset_y }],
                ]
                .into_iter()
            })
        })
    })
}

#[allow(
    clippy::inline_always,
    clippy::match_same_arms,
    clippy::too_many_lines,
    clippy::many_single_char_names
)]
#[inline(always)]
fn generate_hex_offset<T: CoordsFloat>(
    dart: DartIdType,
    [n_x, n_y, _]: [usize; 3],
    [lx, ly, lz]: [T; 3],
) -> Vector3<T> {
    // d = p + 24*x + 24*NX*y + 24*NX*NY*z
    let d = dart as usize;
    let dm = d % 24;
    let dmm = d % (24 * n_x);
    let dmmm = d % (24 * n_x * n_y);
    let p = dm;
    let x = (dmm - dm) / 24;
    let y = (dmmm - dmm) / (24 * n_x);
    let z = (d - dmmm) / (24 * n_x * n_y);
    match p {
        // d1 to d24
        // y- face
        1 | 6 | 17 => Vector3(
            T::from(x).unwrap() * lx,
            T::from(y).unwrap() * ly,
            T::from(z).unwrap() * lz,
        ),
        2 | 5 | 10 => Vector3(
            T::from(x + 1).unwrap() * lx,
            T::from(y).unwrap() * ly,
            T::from(z).unwrap() * lz,
        ),
        3 | 9 | 14 => Vector3(
            T::from(x + 1).unwrap() * lx,
            T::from(y).unwrap() * ly,
            T::from(z + 1).unwrap() * lz,
        ),
        4 | 13 | 18 => Vector3(
            T::from(x).unwrap() * lx,
            T::from(y).unwrap() * ly,
            T::from(z + 1).unwrap() * lz,
        ),
        7 | 20 | 22 => Vector3(
            T::from(x).unwrap() * lx,
            T::from(y + 1).unwrap() * ly,
            T::from(z).unwrap() * lz,
        ),
        8 | 11 | 21 => Vector3(
            T::from(x + 1).unwrap() * lx,
            T::from(y + 1).unwrap() * ly,
            T::from(z).unwrap() * lz,
        ),
        12 | 15 | 0 => Vector3(
            T::from(x + 1).unwrap() * lx,
            T::from(y + 1).unwrap() * ly,
            T::from(z + 1).unwrap() * lz,
        ),
        16 | 19 | 23 => Vector3(
            T::from(x).unwrap() * lx,
            T::from(y + 1).unwrap() * ly,
            T::from(z + 1).unwrap() * lz,
        ),
        _ => unreachable!(),
    }
}

/// Internal grid-building routine
#[allow(clippy::too_many_lines)]
pub fn build_3d_tetgrid<T: CoordsFloat>(
    origin: Vertex3<T>,
    n_cells_per_axis: [usize; 3],
    lengths: [T; 3],
    manager: AttrStorageManager,
) -> CMap3<T> {
    let [n_square_x, n_square_y, n_square_z] = n_cells_per_axis;
    let n_darts = 60 * n_square_x * n_square_y * n_square_z;

    let map: CMap3<T> = CMap3::new_with_undefined_attributes(n_darts, manager);

    // init beta functions
    (1..=n_darts as DartIdType)
        .zip(generate_tet_beta_values(n_cells_per_axis))
        .for_each(|(dart, images)| {
            map.set_betas(dart, images);
        });

    // place vertices
    (1..=n_darts as DartIdType)
        .filter(|d| *d as VertexIdType == map.vertex_id(*d))
        .for_each(|d| {
            let v = origin + generate_tet_offset(d, n_cells_per_axis, lengths);
            map.force_write_vertex(d as VertexIdType, v);
        });

    // check the number of built volumes
    // this is set as debug only because the operation cost scales with map size
    // this can quickly overshadow the exectime of all previous code
    debug_assert_eq!(
        map.iter_volumes().count(),
        n_square_x * n_square_y * n_square_z * 5
    );

    map
}

#[allow(clippy::inline_always, clippy::too_many_lines)]
#[rustfmt::skip]
#[inline(always)]
fn generate_tet_beta_values(
    [n_x, n_y, n_z]: [usize; 3],
) -> impl Iterator<Item = [DartIdType; 4]> {
    // this loop hierarchy yields the value in correct order
    (0..n_z).flat_map(move |iz| {
        (0..n_y).flat_map(move |iy| {
            (0..n_x).flat_map(move |ix| {
                let d1 = (1 + 60 * ix + n_x * 60 * iy + n_x * n_y * 60 * iz) as DartIdType;
                let (    d2 , d3 , d4 , d5 , d6 , d7 , d8 , d9 , d10, d11, d12,
                    d13, d14, d15, d16, d17, d18, d19, d20, d21, d22, d23, d24,
                    d25, d26, d27, d28, d29, d30, d31, d32, d33, d34, d35, d36,
                    d37, d38, d39, d40, d41, d42, d43, d44, d45, d46, d47, d48,
                    d49, d50, d51, d52, d53, d54, d55, d56, d57, d58, d59, d60,
                ) = (        d1 + 1 , d1 + 2 , d1 + 3 , d1 + 4 , d1 + 5 , d1 + 6 , d1 + 7 , d1 + 8 , d1 + 9 ,
                    d1 + 10, d1 + 11, d1 + 12, d1 + 13, d1 + 14, d1 + 15, d1 + 16, d1 + 17, d1 + 18, d1 + 19,
                    d1 + 20, d1 + 21, d1 + 22, d1 + 23, d1 + 24, d1 + 25, d1 + 26, d1 + 27, d1 + 28, d1 + 29,
                    d1 + 30, d1 + 31, d1 + 32, d1 + 33, d1 + 34, d1 + 35, d1 + 36, d1 + 37, d1 + 38, d1 + 39,
                    d1 + 40, d1 + 41, d1 + 42, d1 + 43, d1 + 44, d1 + 45, d1 + 46, d1 + 47, d1 + 48, d1 + 49,
                    d1 + 50, d1 + 51, d1 + 52, d1 + 53, d1 + 54, d1 + 55, d1 + 56, d1 + 57, d1 + 58, d1 + 59, 
                );
                let noffset_x = 60;
                let noffset_y = noffset_x * n_x as DartIdType;
                let noffset_z = noffset_y * n_y as DartIdType;
                let mirror = ((ix + iy + iz) & 1) == 0; // is even

                // beta images of the tetcube (tm)
                if mirror {
                    [
                        // first tet, orthogonal corner facing -x,-y,-z
                        [ d3, d2 , d4 , if iy == 0       { 0 } else { d15 - noffset_y }],
                        [ d1, d3 , d7 , if iy == 0       { 0 } else { d14 - noffset_y }],
                        [ d2, d1 , d10, if iy == 0       { 0 } else { d13 - noffset_y }],
                        [ d6, d5 , d1 , if iz == 0       { 0 } else { d10 - noffset_z }],
                        [ d4, d6 , d12, if iz == 0       { 0 } else { d12 - noffset_z }],
                        [ d5, d4 , d8 , if iz == 0       { 0 } else { d11 - noffset_z }],
                        [ d9, d8 , d2 , d49],
                        [ d7, d9 , d6 , d51],
                        [ d8, d7 , d11, d50],
                        [d12, d11, d3 , if ix == 0       { 0 } else { d28 - noffset_x }],
                        [d10, d12, d9 , if ix == 0       { 0 } else { d30 - noffset_x }],
                        [d11, d10, d5 , if ix == 0       { 0 } else { d29 - noffset_x }],
                        // second tet, orthogonal corner facing +x,+y,-z
                        [d15, d14, d16, if iy == n_y - 1 { 0 } else { d27 + noffset_y }],
                        [d13, d15, d19, if iy == n_y - 1 { 0 } else { d26 + noffset_y }],
                        [d14, d13, d22, if iy == n_y - 1 { 0 } else { d25 + noffset_y }],
                        [d18, d17, d13, if iz == 0       { 0 } else { d46 - noffset_z }],
                        [d16, d18, d24, if iz == 0       { 0 } else { d48 - noffset_z }],
                        [d17, d16, d20, if iz == 0       { 0 } else { d47 - noffset_z }],
                        [d21, d20, d14, d59],
                        [d19, d21, d18, d58],
                        [d20, d19, d23, d60],
                        [d24, d23, d15, if ix == n_x - 1 { 0 } else { d16 + noffset_x }],
                        [d22, d24, d21, if ix == n_x - 1 { 0 } else { d18 + noffset_x }],
                        [d23, d22, d17, if ix == n_x - 1 { 0 } else { d17 + noffset_x }],
                        // third tet, orthogonal corner facing +x,-y,+z
                        [d27, d26, d28, if iy == 0       { 0 } else { d39 - noffset_y }],
                        [d25, d27, d31, if iy == 0       { 0 } else { d38 - noffset_y }],
                        [d26, d25, d34, if iy == 0       { 0 } else { d37 - noffset_y }],
                        [d30, d29, d25, if iz == n_z - 1 { 0 } else { d34 + noffset_z }],
                        [d28, d30, d36, if iz == n_z - 1 { 0 } else { d36 + noffset_z }],
                        [d29, d28, d32, if iz == n_z - 1 { 0 } else { d35 + noffset_z }],
                        [d33, d32, d26, d52],
                        [d31, d33, d30, d54],
                        [d32, d31, d35, d53],
                        [d36, d35, d27, if ix == n_x - 1 { 0 } else { d4  + noffset_x }],
                        [d34, d36, d33, if ix == n_x - 1 { 0 } else { d6  + noffset_x }],
                        [d35, d34, d29, if ix == n_x - 1 { 0 } else { d5  + noffset_x }],
                        // fourth tet, orthogonal corner facing -x,+y,+z
                        [d39, d38, d40, if iy == n_y - 1 { 0 } else { d3  + noffset_y }],
                        [d37, d39, d43, if iy == n_y - 1 { 0 } else { d2  + noffset_y }],
                        [d38, d37, d46, if iy == n_y - 1 { 0 } else { d1  + noffset_y }],
                        [d42, d41, d37, if iz == n_z - 1 { 0 } else { d22 + noffset_z }],
                        [d40, d42, d48, if iz == n_z - 1 { 0 } else { d24 + noffset_z }],
                        [d41, d40, d44, if iz == n_z - 1 { 0 } else { d23 + noffset_z }],
                        [d45, d44, d38, d57],
                        [d43, d45, d42, d56],
                        [d44, d43, d47, d55],
                        [d48, d47, d39, if ix == 0       { 0 } else { d40 - noffset_x }],
                        [d46, d48, d45, if ix == 0       { 0 } else { d42 - noffset_x }],
                        [d47, d46, d41, if ix == 0       { 0 } else { d41 - noffset_x }],
                        // fifth (inner) tet, no orthogonal corner 
                        [d51, d50, d52, d7 ],
                        [d49, d51, d55, d9 ],
                        [d50, d49, d58, d8 ],
                        [d54, d53, d49, d31],
                        [d52, d54, d60, d33],
                        [d53, d52, d56, d32],
                        [d57, d56, d50, d45],
                        [d55, d57, d54, d44],
                        [d56, d55, d59, d43],
                        [d60, d59, d51, d20],
                        [d58, d60, d57, d19],
                        [d59, d58, d53, d21],
                    ]
                    .into_iter()
                } else {
                    [
                        // first tet, orthogonal corner facing -x,-y,+z
                        [ d3, d2 , d4 , if iy == 0       { 0 } else { d39 - noffset_y }],
                        [ d1, d3 , d7 , if iy == 0       { 0 } else { d38 - noffset_y }],
                        [ d2, d1 , d10, if iy == 0       { 0 } else { d37 - noffset_y }],
                        [ d6, d5 , d1 , if ix == 0       { 0 } else { d34 - noffset_x }],
                        [ d4, d6 , d12, if ix == 0       { 0 } else { d36 - noffset_x }],
                        [ d5, d4 , d8 , if ix == 0       { 0 } else { d35 - noffset_x }],
                        [ d9, d8 , d2 , d49],
                        [ d7, d9 , d6 , d51],
                        [ d8, d7 , d11, d50],
                        [d12, d11, d3 , if iz == n_z - 1 { 0 } else { d4 + noffset_z }],
                        [d10, d12, d9 , if iz == n_z - 1 { 0 } else { d6 + noffset_z }],
                        [d11, d10, d5 , if iz == n_z - 1 { 0 } else { d5 + noffset_z }],
                        // second tet, orthogonal corner facing -x,+y,-z
                        [d15, d14, d16, if iy == n_y - 1 { 0 } else { d3  + noffset_y }],
                        [d13, d15, d19, if iy == n_y - 1 { 0 } else { d2  + noffset_y }],
                        [d14, d13, d22, if iy == n_y - 1 { 0 } else { d1  + noffset_y }],
                        [d18, d17, d13, if ix == 0       { 0 } else { d22 - noffset_x }],
                        [d16, d18, d24, if ix == 0       { 0 } else { d24 - noffset_x }],
                        [d17, d16, d20, if ix == 0       { 0 } else { d23 - noffset_x }],
                        [d21, d20, d14, d59],
                        [d19, d21, d18, d58],
                        [d20, d19, d23, d60],
                        [d24, d23, d15, if iz == 0       { 0 } else { d40 - noffset_z }],
                        [d22, d24, d21, if iz == 0       { 0 } else { d42 - noffset_z }],
                        [d23, d22, d17, if iz == 0       { 0 } else { d41 - noffset_z }],
                        // third tet, orthogonal corner facing +x,-y,-z
                        [d27, d26, d28, if iy == 0       { 0 } else { d15 - noffset_y }],
                        [d25, d27, d31, if iy == 0       { 0 } else { d14 - noffset_y }],
                        [d26, d25, d34, if iy == 0       { 0 } else { d13 - noffset_y }],
                        [d30, d29, d25, if ix == n_x - 1 { 0 } else { d10 + noffset_x }],
                        [d28, d30, d36, if ix == n_x - 1 { 0 } else { d12 + noffset_x }],
                        [d29, d28, d32, if ix == n_x - 1 { 0 } else { d11 + noffset_x }],
                        [d33, d32, d26, d52],
                        [d31, d33, d30, d54],
                        [d32, d31, d35, d53],
                        [d36, d35, d27, if iz == 0       { 0 } else { d28 - noffset_z }],
                        [d34, d36, d33, if iz == 0       { 0 } else { d30 - noffset_z }],
                        [d35, d34, d29, if iz == 0       { 0 } else { d29 - noffset_z }],
                        // fourth tet, orthogonal corner facing +x,+y,+z
                        [d39, d38, d40, if iy == n_y - 1 { 0 } else { d27 + noffset_y }],
                        [d37, d39, d43, if iy == n_y - 1 { 0 } else { d26 + noffset_y }],
                        [d38, d37, d46, if iy == n_y - 1 { 0 } else { d25 + noffset_y }],
                        [d42, d41, d37, if ix == n_x - 1 { 0 } else { d46 + noffset_x }],
                        [d40, d42, d48, if ix == n_x - 1 { 0 } else { d48 + noffset_x }],
                        [d41, d40, d44, if ix == n_x - 1 { 0 } else { d47 + noffset_x }],
                        [d45, d44, d38, d57],
                        [d43, d45, d42, d56],
                        [d44, d43, d47, d55],
                        [d48, d47, d39, if iz == n_z - 1 { 0 } else { d16 + noffset_z }],
                        [d46, d48, d45, if iz == n_z - 1 { 0 } else { d18 + noffset_z }],
                        [d47, d46, d41, if iz == n_z - 1 { 0 } else { d17 + noffset_z }],
                        // fifth (inner) tet, no orthogonal corner 
                        [d51, d50, d52, d7 ],
                        [d49, d51, d55, d9 ],
                        [d50, d49, d58, d8 ],
                        [d54, d53, d49, d31],
                        [d52, d54, d60, d33],
                        [d53, d52, d56, d32],
                        [d57, d56, d50, d45],
                        [d55, d57, d54, d44],
                        [d56, d55, d59, d43],
                        [d60, d59, d51, d20],
                        [d58, d60, d57, d19],
                        [d59, d58, d53, d21],
                    ]
                    .into_iter()
                }
            })
        })
    })
}

#[allow(
    clippy::inline_always,
    clippy::match_same_arms,
    clippy::too_many_lines,
    clippy::many_single_char_names
)]
#[inline(always)]
fn generate_tet_offset<T: CoordsFloat>(
    dart: DartIdType,
    [n_x, n_y, _]: [usize; 3],
    [lx, ly, lz]: [T; 3],
) -> Vector3<T> {
    // d = p + 24*x + 24*NX*y + 24*NX*NY*z
    let d = dart as usize;
    let dm = d % 60;
    let dmm = d % (60 * n_x);
    let dmmm = d % (60 * n_x * n_y);
    let p = dm;
    let x = (dmm - dm) / 60;
    let y = (dmmm - dmm) / (60 * n_x);
    let z = (d - dmmm) / (60 * n_x * n_y);
    let mirror = ((x + y + z) & 1) == 0; // is even
    if mirror {
        match p {
            1 | 5 | 10 => Vector3(
                T::from(x).unwrap() * lx,
                T::from(y).unwrap() * ly,
                T::from(z).unwrap() * lz,
            ),
            2 | 4 | 8 | 18 | 21 | 24 | 27 | 31 | 35 | 49 | 53 | 58 => Vector3(
                T::from(x + 1).unwrap() * lx,
                T::from(y).unwrap() * ly,
                T::from(z).unwrap() * lz,
            ),
            3 | 7 | 11 | 26 | 28 | 32 | 42 | 45 | 48 | 50 | 52 | 56 => Vector3(
                T::from(x).unwrap() * lx,
                T::from(y).unwrap() * ly,
                T::from(z + 1).unwrap() * lz,
            ),
            6 | 9 | 12 | 14 | 16 | 20 | 39 | 43 | 47 | 51 | 55 | 59 => Vector3(
                T::from(x).unwrap() * lx,
                T::from(y + 1).unwrap() * ly,
                T::from(z).unwrap() * lz,
            ),
            13 | 17 | 22 => Vector3(
                T::from(x + 1).unwrap() * lx,
                T::from(y + 1).unwrap() * ly,
                T::from(z).unwrap() * lz,
            ),
            15 | 19 | 23 | 30 | 33 | 36 | 38 | 40 | 44 | 54 | 57 | 60 => Vector3(
                T::from(x + 1).unwrap() * lx,
                T::from(y + 1).unwrap() * ly,
                T::from(z + 1).unwrap() * lz,
            ),
            25 | 29 | 34 => Vector3(
                T::from(x + 1).unwrap() * lx,
                T::from(y).unwrap() * ly,
                T::from(z + 1).unwrap() * lz,
            ),
            37 | 41 | 46 => Vector3(
                T::from(x).unwrap() * lx,
                T::from(y + 1).unwrap() * ly,
                T::from(z + 1).unwrap() * lz,
            ),
            _ => unreachable!(),
        }
    } else {
        match p {
            1 | 5 | 10 => Vector3(
                T::from(x).unwrap() * lx,
                T::from(y).unwrap() * ly,
                T::from(z + 1).unwrap() * lz,
            ),
            2 | 4 | 8 | 18 | 21 | 24 | 27 | 31 | 35 | 49 | 53 | 58 => Vector3(
                T::from(x).unwrap() * lx,
                T::from(y).unwrap() * ly,
                T::from(z).unwrap() * lz,
            ),
            3 | 7 | 11 | 26 | 28 | 32 | 42 | 45 | 48 | 50 | 52 | 56 => Vector3(
                T::from(x + 1).unwrap() * lx,
                T::from(y).unwrap() * ly,
                T::from(z + 1).unwrap() * lz,
            ),
            6 | 9 | 12 | 14 | 16 | 20 | 39 | 43 | 47 | 51 | 55 | 59 => Vector3(
                T::from(x).unwrap() * lx,
                T::from(y + 1).unwrap() * ly,
                T::from(z + 1).unwrap() * lz,
            ),
            13 | 17 | 22 => Vector3(
                T::from(x).unwrap() * lx,
                T::from(y + 1).unwrap() * ly,
                T::from(z).unwrap() * lz,
            ),
            15 | 19 | 23 | 30 | 33 | 36 | 38 | 40 | 44 | 54 | 57 | 60 => Vector3(
                T::from(x + 1).unwrap() * lx,
                T::from(y + 1).unwrap() * ly,
                T::from(z).unwrap() * lz,
            ),
            25 | 29 | 34 => Vector3(
                T::from(x + 1).unwrap() * lx,
                T::from(y).unwrap() * ly,
                T::from(z).unwrap() * lz,
            ),
            37 | 41 | 46 => Vector3(
                T::from(x + 1).unwrap() * lx,
                T::from(y + 1).unwrap() * ly,
                T::from(z + 1).unwrap() * lz,
            ),
            _ => unreachable!(),
        }
    }
}
