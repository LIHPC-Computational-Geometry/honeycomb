//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use std::{
    cmp::{max, min},
    collections::{HashMap, VecDeque},
};

use crate::{grisubal::model::Geometry2, GridCellId, IsBoundary};
use honeycomb_core::{CMap2, CMapBuilder, CoordsFloat, DartIdentifier, Vertex2, VertexIdentifier};

use super::model::GeometryVertex;

// ------ CONTENT

macro_rules! left_intersec {
    ($va: ident, $vb: ident, $vdart:ident, $cy: ident) => {{
        let s = ($vdart.x() - $va.x()) / ($vb.x() - $va.x());
        (s, ($vdart.y() - $va.y() - ($vb.y() - $va.y()) * s) / $cy)
    }};
}

macro_rules! right_intersec {
    ($va: ident, $vb: ident, $vdart:ident, $cy: ident) => {{
        let s = ($vdart.x() - $va.x()) / ($vb.x() - $va.x());
        (s, (($vb.y() - $va.y()) * s - ($vdart.y() - $va.y())) / $cy)
    }};
}

macro_rules! down_intersec {
    ($va: ident, $vb: ident, $vdart:ident, $cx: ident) => {{
        let s = ($vdart.y() - $va.y()) / ($vb.y() - $va.y());
        (s, (($vb.x() - $va.x()) * s - ($vdart.x() - $va.x())) / $cx)
    }};
}

macro_rules! up_intersec {
    ($va: ident, $vb: ident, $vdart:ident, $cx: ident) => {{
        let s = ($vdart.y() - $va.y()) / ($vb.y() - $va.y());
        (s, (($vdart.x() - $va.x()) - ($vb.x() - $va.x()) * s) / $cx)
    }};
}

/// Inner building routine.
///
/// This function builds a combinatorial map from the described geometry. The returned
/// map is an adjusted grid that can be clipped in order to keep only part of the mesh.
/// See [`grisubal::Clip`] for more information.
///
/// # Arguments
///
/// - `geometry: &Geometry2<T>` -- Description of the input geometry.
///
/// ## Generics
///
/// - `T: CoordsFloat` -- Floating point type used for coordinate representation.
#[allow(clippy::too_many_lines)]
pub fn build_mesh<T: CoordsFloat>(geometry: &Geometry2<T>, grid_cell_sizes: (T, T)) -> CMap2<T> {
    // build the overlapping grid we'll modify
    let bbox = geometry.bbox();
    let (cx, cy) = grid_cell_sizes; // will need later
    let (nx, ny) = (
        (bbox.max_x / cx).ceil().to_usize().unwrap(),
        (bbox.max_y / cy).ceil().to_usize().unwrap(),
    );
    let ogrid = bbox.overlapping_grid(grid_cell_sizes);
    let mut cmap = CMapBuilder::default()
        .grid_descriptor(ogrid)
        .add_attribute::<IsBoundary>() // will be used for clipping
        .build()
        .expect("E: could not build overlapping grid map");

    // process the geometry

    // STEP 1
    // the aim of this step is to build an exhaustive list of the segments making up
    // the GEOMETRY INTERSECTED WITH THE GRID, i.e. for each segment, if both vertices
    // do not belong to the same cell, we break it into sub-segments until it is the case.

    let mut n_vertices = geometry.vertices.len();
    let mut new_segments = HashMap::with_capacity(geometry.poi.len() * 2); // that *2 has no basis
    geometry.segments.iter().for_each(|&(v1_id, v2_id)| {
        // fetch vertices of the segment
        let (v1, v2) = (&geometry.vertices[v1_id], &geometry.vertices[v2_id]);
        // compute their position in the grid
        // we assume that the origin of the grid is at (0., 0.)
        let (c1, c2) = (
            GridCellId(
                (v1.x() / cx).floor().to_usize().unwrap(),
                (v1.y() / cy).floor().to_usize().unwrap(),
            ),
            GridCellId(
                (v2.x() / cx).floor().to_usize().unwrap(),
                (v2.y() / cy).floor().to_usize().unwrap(),
            ),
        );
        // check neighbor status
        match GridCellId::man_dist(&c1, &c2) {
            // trivial case:
            // v1 & v2 belong to the same cell
            0 => {
                new_segments.insert(
                    if geometry.poi.contains(&v1_id) {
                        GeometryVertex::PoI(v1_id)
                    } else {
                        GeometryVertex::Regular(v1_id)
                    },
                    if geometry.poi.contains(&v2_id) {
                        GeometryVertex::PoI(v2_id)
                    } else {
                        GeometryVertex::Regular(v2_id)
                    },
                );
            }
            // ok case:
            // v1 & v2 belong to neighboring cells
            1 => {
                // fetch base dart the cell of v1
                #[allow(clippy::cast_possible_truncation)]
                let d0 = (1 + 4 * c1.0 + nx * 4 * c1.1) as DartIdentifier;
                // which edge of the cell are we intersecting?
                let diff = GridCellId::diff(&c1, &c2);
                #[rustfmt::skip]
                let (mut t, dart_id) = match diff {
                    // cross left
                    (-1,  0) => {
                        // vertex associated to crossed dart, i.e. top left corner
                        let v_dart = Vertex2::from((
                            T::from(c1.0    ).unwrap() * cx,
                            T::from(c1.1 + 1).unwrap() * cy,
                        ));
                        // call macro
                        let (_, t) = left_intersec!(v1, v2, v_dart, cy);
                        (t, d0 + 3)
                    }
                    // cross right
                    ( 1,  0) => {
                        // vertex associated to crossed dart, i.e. down right corner
                        let v_dart = Vertex2::from((
                            T::from(c1.0 + 1).unwrap() * cx,
                            T::from(c1.1    ).unwrap() * cy,
                        ));
                        let (_, t) = right_intersec!(v1, v2, v_dart, cy);
                        (t, d0 + 1) // adjust for dart direction
                    }
                    // cross down
                    ( 0, -1) => {
                        // vertex associated to crossed dart, i.e. down left corner
                        let v_dart = Vertex2::from((
                            T::from(c1.0).unwrap() * cx,
                            T::from(c1.1).unwrap() * cy,
                        ));
                        let (_, t) = down_intersec!(v1, v2, v_dart, cx);
                        (t, d0)
                    }
                    // cross up
                    ( 0,  1) => {
                        // vertex associated to crossed dart, i.e. up right corner
                        let v_dart = Vertex2::from((
                            T::from(c1.0 + 1).unwrap() * cx,
                            T::from(c1.1 + 1).unwrap() * cy,
                        ));
                        let (_, t) = up_intersec!(v1, v2, v_dart, cx);
                        (t, d0 + 2)
                    }
                    _ => unreachable!(),
                };
                // t is adjusted for dart direction; but it needs to be adjusted according to the edge's direction
                let edge_id = cmap.edge_id(dart_id);
                // works in 2D because edges are 2 darts at most
                if edge_id != dart_id {
                    t = T::one() - t;
                }
                // insert the intersection point & add it to the vertices/poi
                cmap.split_edge(edge_id, Some(t));
                let new_vid = cmap.beta::<1>(dart_id) as VertexIdentifier;
                new_segments.insert(
                    if geometry.poi.contains(&v1_id) {
                        GeometryVertex::PoI(v1_id)
                    } else {
                        GeometryVertex::Regular(v1_id)
                    },
                    GeometryVertex::Intersec(new_vid),
                );
                new_segments.insert(
                    GeometryVertex::Intersec(new_vid),
                    if geometry.poi.contains(&v2_id) {
                        GeometryVertex::PoI(v2_id)
                    } else {
                        GeometryVertex::Regular(v2_id)
                    },
                );
            }
            // highly annoying case:
            // v1 & v2 do not belong to neighboring cell
            _ => {
                // because we're using strait segments (not curves), the manhattan distance gives us
                // the number of cell we're going through to reach v2 from v1, which is equal to the number of
                // additional vertices resulting from intersection with the grid
                // i.e. we're generating i+1 segments
                let diff = GridCellId::diff(&c1, &c2);
                // pure vertical / horizontal traversal are treated separately because `t` is computed directly
                // other cases require adjustment since we'll be computating `t`s over longer segments rather than
                // the edge of a single grid case
                match diff {
                    (i, 0) => {
                        // we can solve the intersection equation
                        // for each vertical edge of the grid we cross (i times)
                        let i_base = c1.0 as isize;
                        let mut vs: VecDeque<GeometryVertex> = if i > 0 {
                            // cross to right => v_dart is the bottom right vertex of the cell
                            let offrange = i_base + 1..=i_base + i;
                            let y_v_dart = T::from(c1.1).unwrap() * cy;
                            offrange
                                .map(|x| {
                                    let x_v_dart = T::from(x).unwrap() * cx;
                                    let v_dart = Vertex2::from((x_v_dart, y_v_dart));
                                    let (_, mut t) = right_intersec!(v1, v2, v_dart, cy);
                                    let d_base = (1 + 4 * (x - 1) + (nx * 4 * c1.1) as isize)
                                        as DartIdentifier;
                                    // adjust t for edge direction
                                    let dart_id = d_base + 1;
                                    let edge_id = cmap.edge_id(dart_id);
                                    // works in 2D because edges are 2 darts at most
                                    if edge_id != dart_id {
                                        t = T::one() - t;
                                    }
                                    cmap.split_edge(edge_id, Some(t));
                                    let new_vid = cmap.beta::<1>(dart_id) as VertexIdentifier;
                                    GeometryVertex::Intersec(new_vid)
                                })
                                .collect()
                        } else {
                            // cross to left  => v_dart is the top left vertex of the cell
                            let offrange = (i_base + 1 + i..=i_base);
                            let y_v_dart = T::from(c1.1 + 1).unwrap() * cy;
                            offrange
                                .map(|x| {
                                    let x_v_dart = T::from(x).unwrap() * cx;
                                    let v_dart = Vertex2::from((x_v_dart, y_v_dart));
                                    let (_, mut t) = left_intersec!(v1, v2, v_dart, cy);
                                    let d_base =
                                        (1 + 4 * x + (nx * 4 * c1.1) as isize) as DartIdentifier;
                                    // adjust t for edge direction
                                    let dart_id = d_base + 3;
                                    let edge_id = cmap.edge_id(dart_id);
                                    // works in 2D because edges are 2 darts at most
                                    if edge_id != dart_id {
                                        t = T::one() - t;
                                    }
                                    cmap.split_edge(edge_id, Some(t));
                                    let new_vid = cmap.beta::<1>(dart_id) as VertexIdentifier;
                                    GeometryVertex::Intersec(new_vid)
                                })
                                .rev() // reverse to preserve v1 to v2 order
                                .collect()
                        };
                        vs.push_front(if geometry.poi.contains(&v1_id) {
                            GeometryVertex::PoI(v1_id)
                        } else {
                            GeometryVertex::Regular(v1_id)
                        });
                        vs.push_back(if geometry.poi.contains(&v2_id) {
                            GeometryVertex::PoI(v2_id)
                        } else {
                            GeometryVertex::Regular(v2_id)
                        });
                        vs.make_contiguous().windows(2).for_each(|seg| {
                            new_segments.insert(seg[0].clone(), seg[1].clone());
                        });
                    }
                    (0, j) => {
                        // we can solve the intersection equation
                        // for each horizontal edge of the grid we cross (j times)
                        let j_base = c1.0 as isize;
                        let mut vs: VecDeque<GeometryVertex> = if j > 0 {
                            // cross to top => v_dart is the top right vertex of the cell
                            let offrange = j_base + 1..=j_base + j;
                            let x_v_dart = T::from(c1.0 + 1).unwrap() * cx;
                            offrange
                                .map(|y| {
                                    let y_v_dart = T::from(y).unwrap() * cy;
                                    let v_dart = Vertex2::from((x_v_dart, y_v_dart));
                                    let (_, mut t) = up_intersec!(v1, v2, v_dart, cx);
                                    let d_base =
                                        (1 + 4 * c1.0 + nx * 4 * y as usize) as DartIdentifier;
                                    // adjust t for edge direction
                                    let dart_id = d_base + 2;
                                    let edge_id = cmap.edge_id(dart_id);
                                    // works in 2D because edges are 2 darts at most
                                    if edge_id != dart_id {
                                        t = T::one() - t;
                                    }
                                    cmap.split_edge(edge_id, Some(t));
                                    let new_vid = cmap.beta::<1>(dart_id) as VertexIdentifier;
                                    GeometryVertex::Intersec(new_vid)
                                })
                                .collect()
                        } else {
                            // cross to bottom  => v_dart is the bottom left vertex of the cell
                            let offrange = (j_base + 1 + j..=j_base);
                            let x_v_dart = T::from(c1.0).unwrap() * cx;
                            offrange
                                .map(|y| {
                                    let y_v_dart = T::from(y).unwrap() * cy;
                                    let v_dart = Vertex2::from((x_v_dart, y_v_dart));
                                    let (_, mut t) = down_intersec!(v1, v2, v_dart, cx);
                                    let d_base =
                                        (1 + 4 * c1.0 + nx * 4 * y as usize) as DartIdentifier;
                                    // adjust t for edge direction
                                    let dart_id = d_base;
                                    let edge_id = cmap.edge_id(dart_id);
                                    // works in 2D because edges are 2 darts at most
                                    if edge_id != dart_id {
                                        t = T::one() - t;
                                    }
                                    cmap.split_edge(edge_id, Some(t));
                                    let new_vid = cmap.beta::<1>(dart_id) as VertexIdentifier;
                                    GeometryVertex::Intersec(new_vid)
                                })
                                .rev() // reverse to preserve v1 to v2 order
                                .collect()
                        };
                        vs.push_front(if geometry.poi.contains(&v1_id) {
                            GeometryVertex::PoI(v1_id)
                        } else {
                            GeometryVertex::Regular(v1_id)
                        });
                        vs.push_back(if geometry.poi.contains(&v2_id) {
                            GeometryVertex::PoI(v2_id)
                        } else {
                            GeometryVertex::Regular(v2_id)
                        });
                        vs.make_contiguous().windows(2).for_each(|seg| {
                            new_segments.insert(seg[0].clone(), seg[1].clone());
                        });
                    }
                    (i, j) => {
                        // most annoying case, once again
                        // in order to process this, we'll consider "minimal extended segments" that we should intersect
                        // we'll compute these intersections (s, t and the intersected dart d), sort results by s to
                        // order segments from v1 to v2, and insert the computed segment similarly to previous cases
                        let i_base = c1.0 as isize;
                        let j_base = c1.1 as isize;
                        let ncx: T = T::from(i.abs() + 1).unwrap() * cx; // length of horizontal segments
                        let ncy = T::from(j.abs() + 1).unwrap() * cy; // length of vertical segments

                        // the first thing to determine is the quadrant in which the segment is oriented
                        // taking v1 as the origin & v2 to define the direction
                        // we don't need to worry about i or j being 0 thanks to the outer & current match
                        match (i.is_positive(), j.is_positive()) {
                            (true, true) => {
                                // directed to top right; we'll intersect either top or right dart of grid cells
                                // vertical segments intersections
                                let intersec_v_data = (i_base + 1..=i_base + i)
                                    .map(|x| {
                                        (
                                            Vertex2::from((
                                                T::from(x).unwrap() * cx, // many xs
                                                T::from(j_base).unwrap(), // min y
                                            )),
                                            x - 1, // cell idx along x; useful later
                                        )
                                    })
                                    .map(|(v_dart, cell_x)| {
                                        let (s, t_extended) = right_intersec!(v1, v2, v_dart, ncy);
                                        // in which y cell do we cross the vertical segment?
                                        let cell_y =
                                            (t_extended * T::from(j.abs() + 1).unwrap()).floor();
                                        // what's the value of t on the crossed segment ?
                                        let t =
                                            (t_extended - cell_y) / T::from(j.abs() + 1).unwrap();
                                        // which dart are we crossing?
                                        let d_base = 1
                                            + 4 * cell_x as usize
                                            + nx * 4 * cell_y.to_usize().unwrap();
                                        let dart_id = (d_base + 1) as DartIdentifier; // base dart + 1
                                        (s, t, dart_id)
                                    });
                                let intersec_h_data = (j_base + 1..=j_base + j)
                                    .map(|y| {
                                        (
                                            Vertex2::from((
                                                T::from(i_base + i).unwrap(), // max x
                                                T::from(y).unwrap() * cy,     // many ys
                                            )),
                                            y - 1, // cell idx along y; useful later
                                        )
                                    })
                                    .map(|(v_dart, cell_y)| {
                                        let (s, t_extended) = up_intersec!(v1, v2, v_dart, ncx);
                                        // in which y cell do we cross the vertical segment?
                                        let cell_x =
                                            (t_extended * T::from(i.abs() + 1).unwrap()).floor();
                                        // what's the value of t on the crossed segment ?
                                        let t =
                                            (t_extended - cell_x) / T::from(i.abs() + 1).unwrap();
                                        // which dart are we crossing?
                                        let d_base = 1
                                            + 4 * cell_x.to_usize().unwrap()
                                            + nx * 4 * cell_y as usize;
                                        let dart_id = (d_base + 2) as DartIdentifier; // base dart + 1
                                        (s, t, dart_id)
                                    });
                                // regroup all intersection data
                                let mut intersec_data = vec![];
                                intersec_data.extend(intersec_v_data);
                                intersec_data.extend(intersec_h_data);
                                // sort by s in order to conserve segment order
                                intersec_data
                                    .sort_by(|(s1, _, _), (s2, _, _)| s1.partial_cmp(s2).unwrap());
                                // collect geometry vertices
                                let mut vs = vec![if geometry.poi.contains(&v1_id) {
                                    GeometryVertex::PoI(v1_id)
                                } else {
                                    GeometryVertex::Regular(v1_id)
                                }];
                                vs.extend(intersec_data.iter_mut().map(|(_, t, dart_id)| {
                                    // insert new vertex in the map
                                    let edge_id = cmap.edge_id(*dart_id);
                                    // works in 2D because edges are 2 darts at most
                                    if edge_id != *dart_id {
                                        *t = T::one() - *t;
                                    }
                                    cmap.split_edge(edge_id, Some(*t));
                                    let new_vid = cmap.beta::<1>(*dart_id) as VertexIdentifier;
                                    GeometryVertex::Intersec(new_vid)
                                }));
                                vs.push(if geometry.poi.contains(&v2_id) {
                                    GeometryVertex::PoI(v2_id)
                                } else {
                                    GeometryVertex::Regular(v2_id)
                                });
                                // insert segments
                                vs.windows(2).for_each(|seg| {
                                    new_segments.insert(seg[0].clone(), seg[1].clone());
                                });
                            }
                            (true, false) => {
                                // directed to bottom right; we'll intersect either bottom or right dart of grid cells
                            }
                            (false, false) => {
                                // directed to bottom left; we'll intersect either bottom or left dart of grid cells
                            }
                            (false, true) => {
                                // directed to top left; we'll intersect either top or left dart of grid cells
                            }
                        }
                    }
                }
            }
        };
    });

    // STEP 2

    // return result
    cmap
}

/// Clipping routine.
///
/// This function takes a map built by [`build_mesh`] and removes cells that model the "inside" of
/// the geometry.
pub fn remove_inner<T: CoordsFloat>(
    cmap2: &mut CMap2<T>,
    geometry: &Geometry2<T>,
    invert_normal_dir: bool,
) {
    todo!()
}

/// Clipping routine
///
/// This function takes a map built by [`build_mesh`] and removes cells that model the "outside" of
/// the geometry.
pub fn remove_outer<T: CoordsFloat>(
    cmap2: &mut CMap2<T>,
    geometry: &Geometry2<T>,
    invert_normal_dir: bool,
) {
    todo!()
}
