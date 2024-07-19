//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use std::{
    cmp::{max, min},
    collections::{HashMap, VecDeque},
    process::id,
};

use crate::{Geometry2, GeometryVertex, GridCellId, IsBoundary, MapEdge};
use honeycomb_core::{CMap2, CMapBuilder, CoordsFloat, DartIdentifier, Vertex2, VertexIdentifier};

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

    // FIXME: THE VERTEX INSERTIONS DUE TO INTERSECTIONS ONLY WORKS WITH A SINGLE INTERSECTION PER EDGE
    // POSSIBLE FIX: DELAY VERTEX INSERTION & USE A `nsplit_edge` METHOD INSTEAD OF `split_edge`

    // STEP 1
    // the aim of this step is to build an exhaustive list of the segments making up
    // the GEOMETRY INTERSECTED WITH THE GRID, i.e. for each segment, if both vertices
    // do not belong to the same cell, we break it into sub-segments until it is the case.

    let new_segments = generate_intersected_segments(&mut cmap, geometry, (nx, ny), (cx, cy));

    // STEP 2
    // now that we have a list of "atomic" (non-dividable) segments, we can use it to build
    // the actual segments that will be inserted into the map.
    // For practical reasons, it is easier to avoid having a PoI as the start or the end of a segment,
    // hence the use of the `MapEdge` structure.

    let edges = generate_edge_data(&mut cmap, geometry, &new_segments);

    // STEP 3
    // now that we have some segments that are directly defined between intersections, we can use some N-maps'
    // properties to easily build the geometry into the map.
    // This part relies heavily on "conventions"; the most important thing to note is that the darts in `MapEdge`
    // instances are very precisely set, and can therefore be used to create all the new connectivities.

    insert_edges_in_map(&mut cmap, &edges);

    // return result
    cmap
}

// --- main kernels steps

#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn generate_intersected_segments<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    geometry: &Geometry2<T>,
    (nx, ny): (usize, usize),
    (cx, cy): (T, T),
) -> HashMap<GeometryVertex, GeometryVertex> {
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
                // fetch base dart of the cell of v1
                #[allow(clippy::cast_possible_truncation)]
                let d_base = (1 + 4 * c1.0 + nx * 4 * c1.1) as DartIdentifier;
                // which edge of the cell are we intersecting?
                let diff = GridCellId::diff(&c1, &c2);
                // which dart does this correspond to?
                #[rustfmt::skip]
                let dart_id = match diff {
                    (-1,  0) => d_base + 3,
                    ( 1,  0) => d_base + 1,
                    ( 0, -1) => d_base    ,
                    ( 0,  1) => d_base + 2,
                    _ => unreachable!(),
                };
                // what's the vertex associated to the dart?
                let v_dart = cmap.vertex(cmap.vertex_id(dart_id)).unwrap();
                // compute relative position of the intersection on the interecting edges
                // `s` is relative to the segment `v1v2`, `t` to the grid's segment (the origin being `v_dart`)
                #[rustfmt::skip]
                let (_s, mut t) = match diff {
                    (-1,  0) => left_intersec!(v1, v2, v_dart, cy),
                    ( 1,  0) => right_intersec!(v1, v2, v_dart, cy),
                    ( 0, -1) => down_intersec!(v1, v2, v_dart, cx),
                    ( 0,  1) => up_intersec!(v1, v2, v_dart, cx),
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
                let new_vid = cmap.beta::<1>(dart_id);
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
                // i.e. we're generating d+1 segments
                let diff = GridCellId::diff(&c1, &c2);
                // pure vertical / horizontal traversal are treated separately because `t` is computed directly
                // other cases require adjustment since we'll be computating `t`s over longer segments rather than
                // the edge of a single grid case
                match diff {
                    (i, 0) => {
                        // we can solve the intersection equation
                        // for each vertical edge of the grid we cross (i times)
                        let i_base = c1.0 as isize;
                        let tmp =
                            (min(i_base + 1, i_base + 1 + i)..=max(i_base + i, i_base)).map(|x| {
                                // cell base dart
                                let d_base =
                                    (1 + 4 * (x - 1) + (nx * 4 * c1.1) as isize) as DartIdentifier;
                                // intersected dart
                                let dart_id = if i.is_positive() {
                                    d_base + 1
                                } else {
                                    d_base + 3
                                };
                                // vertex associated to the intersected dart
                                let v_dart = cmap.vertex(cmap.vertex_id(dart_id)).unwrap();
                                // compute intersection
                                let (_s, mut t) = if i.is_positive() {
                                    right_intersec!(v1, v2, v_dart, cy)
                                } else {
                                    left_intersec!(v1, v2, v_dart, cy)
                                };
                                // adjust t for edge direction
                                let edge_id = cmap.edge_id(dart_id);
                                // works in 2D because edges are 2 darts at most
                                if edge_id != dart_id {
                                    t = T::one() - t;
                                }
                                cmap.split_edge(edge_id, Some(t));
                                let new_vid = cmap.beta::<1>(dart_id);
                                GeometryVertex::Intersec(new_vid)
                            });
                        // because of how the the range is written, we need to reverse the iterator in one case
                        // to keep intersection ordered from v1 to v2 (i.e. ensure the segments we build are correct)
                        let mut vs: VecDeque<GeometryVertex> = if i > 0 {
                            tmp.collect()
                        } else {
                            tmp.rev().collect()
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
                        let tmp =
                            (min(j_base + 1, j_base + 1 + j)..=max(j_base + j, j_base)).map(|y| {
                                // cell base dart
                                let d_base = (1 + 4 * c1.0 + nx * 4 * y as usize) as DartIdentifier;
                                // intersected dart
                                let dart_id = if j.is_positive() { d_base + 2 } else { d_base };
                                // vertex associated to the intersected dart
                                let v_dart = cmap.vertex(cmap.vertex_id(dart_id)).unwrap();
                                // compute intersection
                                let (_s, mut t) = if j.is_positive() {
                                    up_intersec!(v1, v2, v_dart, cx)
                                } else {
                                    down_intersec!(v1, v2, v_dart, cx)
                                };
                                // adjust t for edge direction
                                let edge_id = cmap.edge_id(dart_id);
                                // works in 2D because edges are 2 darts at most
                                if edge_id != dart_id {
                                    t = T::one() - t;
                                }
                                cmap.split_edge(edge_id, Some(t));
                                let new_did = cmap.beta::<1>(dart_id);
                                GeometryVertex::Intersec(new_did)
                            });
                        // because of how the the range is written, we need to reverse the iterator in one case
                        // to keep intersection ordered from v1 to v2 (i.e. ensure the segments we build are correct)
                        let mut vs: VecDeque<GeometryVertex> = if j > 0 {
                            tmp.collect()
                        } else {
                            tmp.rev().collect()
                        };
                        // complete the vertex list
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
                        // insert new segments
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
                                    let new_vid = cmap.beta::<1>(*dart_id);
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
                                let intersec_h_data = (j_base + 1 + j..=j_base)
                                    .map(|y| {
                                        (
                                            Vertex2::from((
                                                T::from(i_base).unwrap(), // min x
                                                T::from(y).unwrap() * cy, // many ys
                                            )),
                                            y - 1, // cell idx along y; useful later
                                        )
                                    })
                                    .map(|(v_dart, cell_y)| {
                                        let (s, t_extended) = down_intersec!(v1, v2, v_dart, ncx);
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
                                        let dart_id = d_base as DartIdentifier; // base dart + 1
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
                                    let new_vid = cmap.beta::<1>(*dart_id);
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
                            (false, false) => {
                                // directed to bottom left; we'll intersect either bottom or left dart of grid cells
                                // vertical segments intersections
                                let intersec_v_data = (i_base + 1 + i..=i_base)
                                    .map(|x| {
                                        (
                                            Vertex2::from((
                                                T::from(x).unwrap() * cx,     // many xs
                                                T::from(j_base + j).unwrap(), // max y
                                            )),
                                            x - 1, // cell idx along x; useful later
                                        )
                                    })
                                    .map(|(v_dart, cell_x)| {
                                        let (s, t_extended) = left_intersec!(v1, v2, v_dart, ncy);
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
                                        let dart_id = (d_base + 3) as DartIdentifier; // base dart + 1
                                        (s, t, dart_id)
                                    });
                                let intersec_h_data = (j_base + 1 + j..=j_base)
                                    .map(|y| {
                                        (
                                            Vertex2::from((
                                                T::from(i_base).unwrap(), // min x
                                                T::from(y).unwrap() * cy, // many ys
                                            )),
                                            y - 1, // cell idx along y; useful later
                                        )
                                    })
                                    .map(|(v_dart, cell_y)| {
                                        let (s, t_extended) = down_intersec!(v1, v2, v_dart, ncx);
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
                                        let dart_id = d_base as DartIdentifier; // base dart + 1
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
                                    let new_vid = cmap.beta::<1>(*dart_id);
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
                            (false, true) => {
                                // directed to top left; we'll intersect either top or left dart of grid cells
                                // vertical segments intersections
                                let intersec_v_data = (i_base + 1 + i..=i_base)
                                    .map(|x| {
                                        (
                                            Vertex2::from((
                                                T::from(x).unwrap() * cx,     // many xs
                                                T::from(j_base + j).unwrap(), // max y
                                            )),
                                            x - 1, // cell idx along x; useful later
                                        )
                                    })
                                    .map(|(v_dart, cell_x)| {
                                        let (s, t_extended) = left_intersec!(v1, v2, v_dart, ncy);
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
                                        let dart_id = (d_base + 3) as DartIdentifier; // base dart + 1
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
                                    let new_vid = cmap.beta::<1>(*dart_id);
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
                        }
                    }
                }
            }
        };
    });
    new_segments
}

fn generate_edge_data<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    geometry: &Geometry2<T>,
    new_segments: &HashMap<GeometryVertex, GeometryVertex>,
) -> Vec<MapEdge> {
    new_segments
        .iter()
        .filter(|(k, _)| matches!(k, GeometryVertex::Intersec(_)))
        .map(|(start, v)| {
            let mut end = v;
            let mut intermediates = Vec::new();
            // while we land on regular vertices, go to the next
            while !matches!(end, GeometryVertex::Intersec(_)) {
                match end {
                    GeometryVertex::PoI(vid) => {
                        // insert the PoI in the map; create some darts to go with it
                        let v = geometry.vertices[*vid];
                        let d = cmap.add_free_darts(2);
                        cmap.two_link(d, d + 1);
                        cmap.insert_vertex(d as VertexIdentifier, v);
                        // save intermediate & update end point
                        intermediates.push(d);
                        end = &new_segments[end];
                    }
                    GeometryVertex::Regular(_) => {
                        // skip; update end point
                        end = &new_segments[end];
                    }
                    GeometryVertex::Intersec(_) => unreachable!(), // outer while should prevent this from happening
                }
            }
            let GeometryVertex::Intersec(d_start) = start else {
                // unreachable due to filter
                unreachable!();
            };
            let GeometryVertex::Intersec(d_end) = end else {
                // unreachable due to while block
                unreachable!()
            };

            // the data in this structure can be used to entirely deduce the new connections that should be made
            // at STEP 3
            MapEdge {
                start: cmap.beta::<2>(*d_start), // dart locality shenanigans
                intermediates,
                end: *d_end,
            }
        })
        .collect()
}

fn insert_edges_in_map<T: CoordsFloat>(cmap: &mut CMap2<T>, edges: &[MapEdge]) {
    for MapEdge {
        start,
        intermediates,
        end,
    } in edges
    {
        // remove deprecated connectivities & save what data is necessary
        let b1_start_old = cmap.beta::<1>(*start);
        let b0_end_old = cmap.beta::<0>(*end);
        cmap.one_unlink(*start);
        cmap.one_unlink(b0_end_old);
        let d_new = cmap.add_free_darts(2);
        let b2_d_new = d_new + 1;
        cmap.two_link(d_new, b2_d_new);

        // rebuild
        cmap.one_link(*start, d_new);
        cmap.one_link(b2_d_new, b1_start_old);
        if intermediates.is_empty() {
            // new darts link directly to the end
            cmap.one_link(d_new, *end);
            cmap.one_link(b0_end_old, b2_d_new);
        } else {
            // we need to play with intermediates & windows
            // start to first intermediate; expect should not happen due to if statement
            let di_first = intermediates.first().expect("E: unreachable");
            cmap.one_sew(d_new, *di_first);
            // intermediate to intermediate
            intermediates.windows(2).for_each(|ds| {
                let &[di1, di2] = ds else { unreachable!() };
                cmap.one_sew(di1, di2);
            });
            // last intermediate to end; last may be the same as first
            let di_last = intermediates.last().expect("E: unreachable");
            cmap.one_link(*di_last, *end);
            cmap.one_link(b0_end_old, cmap.beta::<2>(*di_last));
        }
    }
}

// --- clipping

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
