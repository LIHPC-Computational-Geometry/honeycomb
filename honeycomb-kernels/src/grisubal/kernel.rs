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

#[cfg(feature = "profiling")]
use super::{Section, TIMERS};
use crate::grisubal::model::{Boundary, Geometry2, GeometryVertex, MapEdge};
use crate::{grisubal::grid::GridCellId, splits::splitn_edge_no_alloc};
use honeycomb_core::prelude::{
    CMap2, CMapBuilder, CoordsFloat, DartIdentifier, EdgeIdentifier, GridDescriptor, Vertex2,
    NULL_DART_ID,
};

// ------ CONTENT

macro_rules! make_geometry_vertex {
    ($g: ident, $vid: ident) => {
        if $g.poi.contains(&$vid) {
            GeometryVertex::PoI($vid)
        } else {
            GeometryVertex::Regular($vid)
        }
    };
}

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

#[cfg(feature = "profiling")]
macro_rules! unsafe_time_section {
    ($inst: ident, $sec: expr) => {
        unsafe {
            TIMERS[$sec as usize] = Some($inst.elapsed());
            $inst = std::time::Instant::now();
        }
    };
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
pub fn build_mesh<T: CoordsFloat>(
    geometry: &Geometry2<T>,
    [cx, cy]: [T; 2],
    [nx, ny]: [usize; 2],
    origin: Vertex2<T>,
) -> CMap2<T> {
    #[cfg(feature = "profiling")]
    let mut instant = std::time::Instant::now();

    // compute grid characteristics
    // build grid descriptor
    let ogrid = GridDescriptor::default()
        .n_cells_x(nx)
        .n_cells_y(ny)
        .len_per_cell_x(cx)
        .len_per_cell_y(cy)
        .origin(origin);

    // build initial map
    let mut cmap = CMapBuilder::default()
        .grid_descriptor(ogrid)
        .add_attribute::<Boundary>() // will be used for clipping
        .build()
        .expect("E: unreachable"); // urneachable because grid dims are valid

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::BuildMeshInit);

    // process the geometry

    // STEP 1
    // the aim of this step is to build an exhaustive list of the segments making up
    // the GEOMETRY INTERSECTED WITH THE GRID, i.e. for each segment, if both vertices
    // do not belong to the same cell, we break it into sub-segments until it is the case.

    let (new_segments, intersection_metadata) =
        generate_intersection_data(&cmap, geometry, [nx, ny], [cx, cy], origin);

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::BuildMeshIntersecData);

    // STEP 1.5
    // precompute stuff to
    // - parallelize step 2
    // - make step 2 and step 3 independent from each other

    let n_intersec = intersection_metadata.len();
    let (edge_intersec, dart_slices) =
        group_intersections_per_edge(&mut cmap, intersection_metadata);
    let intersection_darts = compute_intersection_ids(n_intersec, &edge_intersec, &dart_slices);

    // STEP 2
    // insert the intersection vertices into the map & recover their encoding dart. The output Vec has consistent
    // indexing with the input Vec, meaning that indices in GeometryVertex::Intersec instances are still valid.

    insert_intersections(&mut cmap, &edge_intersec, &dart_slices);

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::BuildMeshInsertIntersec);

    // STEP 3
    // now that we have a list of "atomic" (non-dividable) segments, we can use it to build the actual segments that
    // will be inserted into the map. Intersections serve as anchor points for the new segments while PoI make up
    // "intermediate" points of segments.

    let edges = generate_edge_data(&cmap, geometry, &new_segments, &intersection_darts);

    #[cfg(feature = "profiling")]
    unsafe_time_section!(instant, Section::BuildMeshEdgeData);

    // STEP 4
    // now that we have some segments that are directly defined between intersections, we can use some N-maps'
    // properties to easily build the geometry into the map.
    // This part relies heavily on "conventions"; the most important thing to note is that the darts in `MapEdge`
    // instances are very precisely set, and can therefore be used to create all the new connectivities.

    insert_edges_in_map(&mut cmap, &edges);

    #[cfg(feature = "profiling")]
    unsafe {
        TIMERS[Section::BuildMeshInsertEdge as usize] = Some(instant.elapsed());
    }

    // return result
    cmap
}

// --- type aliases for clarity

pub type Segments = HashMap<GeometryVertex, GeometryVertex>;

pub type IntersectionsPerEdge<T> = HashMap<EdgeIdentifier, Vec<(usize, T, DartIdentifier)>>;

pub type DartSlices = Vec<Vec<DartIdentifier>>;

// --- main kernels steps

#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub(super) fn generate_intersection_data<T: CoordsFloat>(
    cmap: &CMap2<T>,
    geometry: &Geometry2<T>,
    [nx, _ny]: [usize; 2],
    [cx, cy]: [T; 2],
    origin: Vertex2<T>,
) -> (Segments, Vec<(DartIdentifier, T)>) {
    let tmp: Vec<_> = geometry
        .segments
        .iter()
        .map(|&(v1_id, v2_id)| {
            // fetch vertices of the segment
            let Vertex2(ox, oy) = origin;
            let (v1, v2) = (&geometry.vertices[v1_id], &geometry.vertices[v2_id]);
            // compute their position in the grid
            // we assume that the origin of the grid is at (0., 0.)
            let (c1, c2) = (
                GridCellId(
                    ((v1.x() - ox) / cx).floor().to_usize().unwrap(),
                    ((v1.y() - oy) / cy).floor().to_usize().unwrap(),
                ),
                GridCellId(
                    ((v2.x() - ox) / cx).floor().to_usize().unwrap(),
                    ((v2.y() - oy) / cy).floor().to_usize().unwrap(),
                ),
            );
            (
                GridCellId::man_dist(&c1, &c2),
                GridCellId::diff(&c1, &c2),
                v1,
                v2,
                v1_id,
                v2_id,
                c1,
            )
        })
        .collect();
    // total number of intersection
    let n_intersec: usize = tmp.iter().map(|(dist, _, _, _, _, _, _)| dist).sum();
    // we're using the prefix sum to compute an offset from the start. that's why we need a 0 at the front
    // we'll cut off the last element later
    let prefix_sum: Vec<usize> = [0]
        .iter()
        .chain(tmp.iter().map(|(dist, _, _, _, _, _, _)| dist))
        .scan(0, |state, &dist| {
            *state += dist;
            Some(*state)
        })
        .collect();
    // preallocate the intersection vector
    let mut intersection_metadata = vec![(NULL_DART_ID, T::nan()); n_intersec];

    let new_segments: Segments = tmp.iter().zip(prefix_sum[..prefix_sum.len()-1].iter()).flat_map(|(&(dist, diff, v1, v2, v1_id, v2_id, c1), start)| {
        let transform = Box::new(|seg: &[GeometryVertex]| {
            assert_eq!(seg.len(), 2);
            (seg[0].clone(), seg[1].clone())
        });
        // check neighbor status
        match dist {
            // trivial case:
            // v1 & v2 belong to the same cell
            0 => {
                vec![(make_geometry_vertex!(geometry, v1_id), make_geometry_vertex!(geometry, v2_id))]
            }
            // ok case:
            // v1 & v2 belong to neighboring cells
            1 => {
                // fetch base dart of the cell of v1
                #[allow(clippy::cast_possible_truncation)]
                let d_base = (1 + 4 * c1.0 + nx * 4 * c1.1) as DartIdentifier;
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
                let v_dart = cmap
                    .vertex(cmap.vertex_id(dart_id))
                    .expect("E: found a topological vertex with no associated coordinates");
                // compute relative position of the intersection on the interecting edges
                // `s` is relative to the segment `v1v2`, `t` to the grid's segment (the origin being `v_dart`)
                #[rustfmt::skip]
                let (_s, t) = match diff {
                    (-1,  0) => left_intersec!(v1, v2, v_dart, cy),
                    ( 1,  0) => right_intersec!(v1, v2, v_dart, cy),
                    ( 0, -1) => down_intersec!(v1, v2, v_dart, cx),
                    ( 0,  1) => up_intersec!(v1, v2, v_dart, cx),
                    _ => unreachable!(),
                };

                let id = *start;
                intersection_metadata[id] = (dart_id, t);

                vec![
                    (make_geometry_vertex!(geometry, v1_id), GeometryVertex::Intersec(id)),
                    (GeometryVertex::Intersec(id), make_geometry_vertex!(geometry, v2_id)),
                ]
            }
            // highly annoying case:
            // v1 & v2 do not belong to neighboring cell
            _ => {
                // pure vertical / horizontal traversal are treated separately because it ensures we're not trying
                // to compute intersections of parallel segments (which results at best in a division by 0)
                let i_ids = *start..*start+dist;
                match diff {
                    (i, 0) => {
                        // we can solve the intersection equation
                        // for each vertical edge of the grid we cross (i times)
                        let i_base = c1.0 as isize;
                        let tmp =
                            // the range is either
                            // i > 0: i_base..i_base + i
                            // or
                            // i < 0: i_base + 1 + i..i_base + 1
                            (min(i_base, i_base + 1 + i)..max(i_base + i, i_base + 1)).zip(i_ids).map(|(x, id)| {
                                // cell base dart
                                let d_base =
                                    (1 + 4 * x + (nx * 4 * c1.1) as isize) as DartIdentifier;
                                // intersected dart
                                let dart_id = if i.is_positive() {
                                    d_base + 1
                                } else {
                                    d_base + 3
                                };
                                // vertex associated to the intersected dart
                                let v_dart = cmap.vertex(cmap.vertex_id(dart_id))
                                    .expect("E: found a topological vertex with no associated coordinates");
                                // compute intersection
                                let (_s, t) = if i.is_positive() {
                                    right_intersec!(v1, v2, v_dart, cy)
                                } else {
                                    left_intersec!(v1, v2, v_dart, cy)
                                };

                                intersection_metadata[id] = (dart_id, t);

                                GeometryVertex::Intersec(id)
                            });

                        // because of how the range is written, we need to reverse the iterator in one case
                        // to keep intersection ordered from v1 to v2 (i.e. ensure the segments we build are correct)
                        let mut vs: VecDeque<GeometryVertex> = if i > 0 {
                            tmp.collect()
                        } else {
                            tmp.rev().collect()
                        };

                        // complete the vertex list
                        vs.push_front(make_geometry_vertex!(geometry, v1_id));
                        vs.push_back(make_geometry_vertex!(geometry, v2_id));

                        vs.make_contiguous()
                            .windows(2)
                            .map(transform)
                            .collect::<Vec<_>>()
                    }
                    (0, j) => {
                        // we can solve the intersection equation
                        // for each horizontal edge of the grid we cross (j times)
                        let j_base = c1.1 as isize;
                        let tmp =
                            // the range is either
                            // j > 0: j_base..j_base + j
                            // or
                            // j < 0: j_base + 1 + j..j_base + 1
                            (min(j_base, j_base + 1 + j)..max(j_base + j, j_base + 1)).zip(i_ids).map(|(y, id)| {
                                // cell base dart
                                let d_base = (1 + 4 * c1.0 + nx * 4 * y as usize) as DartIdentifier;
                                // intersected dart
                                let dart_id = if j.is_positive() { d_base + 2 } else { d_base };
                                // vertex associated to the intersected dart
                                let v_dart = cmap.vertex(cmap.vertex_id(dart_id))
                                    .expect("E: found a topological vertex with no associated coordinates");
                                // compute intersection
                                let (_s, t) = if j.is_positive() {
                                    up_intersec!(v1, v2, v_dart, cx)
                                } else {
                                    down_intersec!(v1, v2, v_dart, cx)
                                };

                                intersection_metadata[id] = (dart_id, t);

                                GeometryVertex::Intersec(id)
                            });

                        // because of how the range is written, we need to reverse the iterator in one case
                        // to keep intersection ordered from v1 to v2 (i.e. ensure the segments we build are correct)
                        let mut vs: VecDeque<GeometryVertex> = if j > 0 {
                            tmp.collect()
                        } else {
                            tmp.rev().collect()
                        };

                        // complete the vertex list
                        vs.push_front(make_geometry_vertex!(geometry, v1_id));
                        vs.push_back(make_geometry_vertex!(geometry, v2_id));

                        vs.make_contiguous()
                            .windows(2)
                            .map(transform)
                            .collect::<Vec<_>>()
                    }
                    (i, j) => {
                        // in order to process this, we'll consider a "sub-grid" & use the direction of the segment to
                        // deduce which pair of dart we are supposed to intersect
                        // we also have to consider corner traversal; this corresponds to intersecting both darts of
                        // the pair at respective relative positions 1 and 0 (or 0 and 1)
                        let i_base = c1.0 as isize;
                        let j_base = c1.1 as isize;
                        let i_cell_range = min(i_base, i_base + i)..=max(i_base + i, i_base);
                        let j_cell_range = min(j_base, j_base + j)..=max(j_base + j, j_base);
                        let subgrid_cells =
                            i_cell_range.flat_map(|x| j_cell_range.clone().map(move |y| (x, y)));

                        let mut intersec_data: Vec<(T, T, DartIdentifier)> = subgrid_cells
                            .map(|(x, y)| {
                                // cell base dart
                                let d_base = (1 + 4 * x + nx as isize * 4 * y) as DartIdentifier;
                                // (potentially) intersected darts
                                let vdart_id = if i.is_positive() {
                                    d_base + 1
                                } else {
                                    d_base + 3
                                };
                                let hdart_id = if j.is_positive() { d_base + 2 } else { d_base };
                                // associated vertices
                                let v_vdart = cmap.vertex(cmap.vertex_id(vdart_id))
                                    .expect("E: found a topological vertex with no associated coordinates");
                                let v_hdart = cmap.vertex(cmap.vertex_id(hdart_id))
                                    .expect("E: found a topological vertex with no associated coordinates");
                                // compute (potential) intersections
                                let v_coeffs = if i.is_positive() {
                                    right_intersec!(v1, v2, v_vdart, cy)
                                } else {
                                    left_intersec!(v1, v2, v_vdart, cy)
                                };
                                let h_coeffs = if j.is_positive() {
                                    up_intersec!(v1, v2, v_hdart, cx)
                                } else {
                                    down_intersec!(v1, v2, v_hdart, cx)
                                };

                                (hdart_id, vdart_id, v_coeffs, h_coeffs)
                            })
                            .filter_map(|(hdart_id, vdart_id, (vs, vt), (hs, ht))| {
                                let zero = T::zero();
                                let one = T::one();
                                // there is one corner intersection to check per (i, j) quadrant
                                match (i.is_positive(), j.is_positive()) {
                                    // check
                                    (true, true) | (false, false) => {
                                        if ((vt - one).abs() < T::epsilon())
                                            && (ht.abs() < T::epsilon())
                                        {
                                            return Some((hs, zero, hdart_id));
                                        }
                                    }
                                    (false, true) | (true, false) => {
                                        if (vt.abs() < T::epsilon())
                                            && ((ht - one).abs() < T::epsilon())
                                        {
                                            return Some((vs, zero, vdart_id));
                                        }
                                    }
                                }

                                // we can deduce if and which side is intersected using s and t values
                                // these should be comprised strictly between 0 and 1 for regular intersections
                                if (T::epsilon() <= vs)
                                    & (vs <= one - T::epsilon())
                                    & (T::epsilon() <= vt)
                                    & (vt <= one - T::epsilon())
                                {
                                    return Some((vs, vt, vdart_id)); // intersect vertical side
                                }
                                if (T::epsilon() < hs)
                                    & (hs <= one - T::epsilon())
                                    & (T::epsilon() <= ht)
                                    & (ht <= one - T::epsilon())
                                {
                                    return Some((hs, ht, hdart_id)); // intersect horizontal side
                                }

                                // intersect none; this is possible since we're looking at cells of a subgrid,
                                // not following through the segment's intersections
                                None
                            })
                            .collect();

                        // sort intersections from v1 to v2
                        intersec_data.retain(|(s, _, _)| (T::zero() <= *s) && (*s <= T::one()));
                        // panic unreachable because of the retain above; there's no s s.t. s == NaN
                        intersec_data.sort_by(|(s1, _, _), (s2, _, _)| s1.partial_cmp(s2)
                            .expect("E: unreachable"));

                        // collect geometry vertices
                        let mut vs = vec![make_geometry_vertex!(geometry, v1_id)];
                        vs.extend(intersec_data.iter_mut().zip(i_ids).map(|((_, t, dart_id), id)| {
                            if t.is_zero() {
                                // we assume that the segment fully goes through the corner and does not land exactly
                                // on it, this allows us to compute directly the dart from which the next segment
                                // should start: the one incident to the vertex in the opposite quadrant

                                // in that case, the preallocated intersection metadata slot will stay as (0, Nan)
                                // this is ok, we can simply ignore the entry when processing the data later

                                let dart_in = *dart_id;
                                GeometryVertex::IntersecCorner(dart_in)
                            } else {
                                intersection_metadata[id] = (*dart_id, *t);

                                GeometryVertex::Intersec(id)
                            }
                        }));

                        vs.push(make_geometry_vertex!(geometry, v2_id));

                        vs.windows(2)
                            .map(transform)
                            .collect::<Vec<_>>()
                    }
                }
            }
        }
    }).collect();
    (new_segments, intersection_metadata)
}

pub(super) fn group_intersections_per_edge<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    intersection_metadata: Vec<(DartIdentifier, T)>,
) -> (IntersectionsPerEdge<T>, DartSlices) {
    // group intersection data per edge, and associate an ID to each
    let mut edge_intersec: HashMap<EdgeIdentifier, Vec<(usize, T, DartIdentifier)>> =
        HashMap::new();
    intersection_metadata
        .into_iter()
        .filter(|(_, t)| !t.is_nan())
        .enumerate()
        .for_each(|(idx, (dart_id, mut t))| {
            // classify intersections per edge_id & adjust t if  needed
            let edge_id = cmap.edge_id(dart_id);
            // condition works in 2D because edges are 2 darts at most
            if edge_id != dart_id {
                t = T::one() - t;
            }
            if let Some(storage) = edge_intersec.get_mut(&edge_id) {
                // not the first intersction with this given edge
                storage.push((idx, t, dart_id));
            } else {
                // first intersction with this given edge
                edge_intersec.insert(edge_id, vec![(idx, t, dart_id)]);
            }
        });

    // sort per t for later
    for vs in edge_intersec.values_mut() {
        // panic unreachable because t s.t. t == NaN have been filtered previously
        vs.sort_by(|(_, t1, _), (_, t2, _)| t1.partial_cmp(t2).expect("E: unreachable"));
    }

    // prealloc darts that will be used for vertex insertion
    let n_darts_per_seg: Vec<_> = edge_intersec.values().map(|vs| 2 * vs.len()).collect();
    let n_tot: usize = n_darts_per_seg.iter().sum();
    let tmp = cmap.add_free_darts(n_tot) as usize;
    // the prefix sum gives an offset that corresponds to the starting index of each slice, minus
    // the location of the allocated dart block (given by `tmp`)
    // end of the slice is deduced using these values and the number of darts the current seg needs
    let prefix_sum: Vec<usize> = n_darts_per_seg
        .iter()
        .enumerate()
        .map(|(i, _)| (0..i).map(|idx| n_darts_per_seg[idx]).sum())
        .collect();
    #[allow(clippy::cast_possible_truncation)]
    let dart_slices: Vec<Vec<DartIdentifier>> = n_darts_per_seg
        .iter()
        .zip(prefix_sum.iter())
        .map(|(n_d, start)| {
            ((tmp + start) as DartIdentifier..(tmp + start + n_d) as DartIdentifier)
                .collect::<Vec<_>>()
        })
        .collect();

    (edge_intersec, dart_slices)
}

pub(super) fn insert_intersections<T: CoordsFloat>(
    cmap: &mut CMap2<T>,
    edge_intersec: &IntersectionsPerEdge<T>,
    dart_slices: &DartSlices,
) {
    for ((edge_id, vs), new_darts) in edge_intersec.iter().zip(dart_slices.iter()) {
        let _ = splitn_edge_no_alloc(
            cmap,
            *edge_id,
            new_darts,
            &vs.iter().map(|(_, t, _)| *t).collect::<Vec<_>>(),
        );
    }
}

pub(super) fn compute_intersection_ids<T: CoordsFloat>(
    n_intersec: usize,
    edge_intersec: &IntersectionsPerEdge<T>,
    dart_slices: &DartSlices,
) -> Vec<DartIdentifier> {
    let mut res = vec![NULL_DART_ID; n_intersec];
    for ((edge_id, vs), new_darts) in edge_intersec.iter().zip(dart_slices.iter()) {
        // order should be consistent between collection because of the sort_by call
        let hl = new_darts.len() / 2; // half-length; also equal to n_intermediate
        let fh = &new_darts[..hl]; // first half;  used for the side of edge id
        let sh = &new_darts[hl..]; // second half; used for the opposite side
        for (i, (id, _, old_dart_id)) in vs.iter().enumerate() {
            // readjust according to intersection side
            res[*id] = if *old_dart_id == *edge_id {
                fh[i]
            } else {
                sh[hl - 1 - i]
            };
        }
    }
    res
}

pub(super) fn generate_edge_data<T: CoordsFloat>(
    cmap: &CMap2<T>,
    geometry: &Geometry2<T>,
    new_segments: &Segments,
    intersection_darts: &[DartIdentifier],
) -> Vec<MapEdge<T>> {
    new_segments
        .iter()
        .filter(|(k, _)| {
            matches!(
                k,
                GeometryVertex::Intersec(_) | GeometryVertex::IntersecCorner(..)
            )
        })
        .map(|(start, v)| {
            let mut end = v;
            let mut intermediates = Vec::new();
            // while we land on regular vertices, go to the next
            while !matches!(
                end,
                GeometryVertex::Intersec(_) | GeometryVertex::IntersecCorner(_)
            ) {
                match end {
                    GeometryVertex::PoI(vid) => {
                        // save the PoI as an intermediate & update end point
                        intermediates.push(geometry.vertices[*vid]);
                        end = &new_segments[end];
                    }
                    GeometryVertex::Regular(_) => {
                        // skip; update end point
                        end = &new_segments[end];
                    }
                    GeometryVertex::Intersec(_) | GeometryVertex::IntersecCorner(..) => {
                        unreachable!() // outer while should prevent this from happening
                    }
                }
            }

            let d_start = match start {
                GeometryVertex::Intersec(d_start_idx) => {
                    cmap.beta::<2>(intersection_darts[*d_start_idx])
                }
                GeometryVertex::IntersecCorner(d_in) => {
                    cmap.beta::<2>(cmap.beta::<1>(cmap.beta::<2>(*d_in)))
                }
                _ => unreachable!(), // unreachable due to filter
            };
            let d_end = match end {
                GeometryVertex::Intersec(d_end_idx) => intersection_darts[*d_end_idx],
                GeometryVertex::IntersecCorner(d_in) => *d_in,
                _ => unreachable!(), // unreachable due to filter
            };

            // the data in this structure can be used to entirely deduce the new connections that should be made
            // at STEP 3

            MapEdge {
                start: d_start,
                intermediates,
                end: d_end,
            }
        })
        .collect()
}

pub(super) fn insert_edges_in_map<T: CoordsFloat>(cmap: &mut CMap2<T>, edges: &[MapEdge<T>]) {
    // FIXME: minimize allocs & redundant operations
    // prealloc all darts needed
    let n_darts_per_seg: Vec<_> = edges
        .iter()
        .map(|e| 2 + 2 * e.intermediates.len())
        .collect();
    let n_tot: usize = n_darts_per_seg.iter().sum();
    let tmp = cmap.add_free_darts(n_tot) as usize;
    // the prefix sum gives an offset that corresponds to the starting index of each slice, minus
    // the location of the allocated dart block (given by `tmp`)
    // end of the slice is deduced using these values and the number of darts the current seg needs
    let prefix_sum: Vec<usize> = n_darts_per_seg
        .iter()
        .enumerate()
        .map(|(i, _)| (0..i).map(|idx| n_darts_per_seg[idx]).sum())
        .collect();
    #[allow(clippy::cast_possible_truncation)]
    let dart_slices: Vec<Vec<DartIdentifier>> = n_darts_per_seg
        .iter()
        .zip(prefix_sum.iter())
        .map(|(n_d, start)| {
            ((tmp + start) as DartIdentifier..(tmp + start + n_d) as DartIdentifier)
                .collect::<Vec<_>>()
        })
        .collect();

    // insert new edges
    for (
        MapEdge {
            start,
            intermediates,
            end,
        },
        dslice,
    ) in edges.iter().zip(dart_slices.iter())
    {
        // remove deprecated connectivities & save what data is necessary
        let b1_start_old = cmap.beta::<1>(*start);
        let b0_end_old = cmap.beta::<0>(*end);
        cmap.one_unlink(*start);
        cmap.one_unlink(b0_end_old);

        let &[d_new, b2_d_new] = &dslice[0..2] else {
            unreachable!()
        };
        cmap.two_link(d_new, b2_d_new);

        // rebuild - this is the final construct if there are no intermediates
        cmap.one_link(*start, d_new);
        cmap.one_link(b2_d_new, b1_start_old);
        cmap.one_link(d_new, *end);
        cmap.one_link(b0_end_old, b2_d_new);

        if !intermediates.is_empty() {
            // create the topology components
            let edge_id = cmap.edge_id(d_new);
            let new_darts = &dslice[2..];
            let _ = splitn_edge_no_alloc(
                cmap,
                edge_id,
                new_darts,
                &vec![T::from(0.5).unwrap(); intermediates.len()],
            );
            // replace placeholder vertices
            let mut dart_id = cmap.beta::<1>(edge_id as DartIdentifier);
            for v in intermediates {
                let vid = cmap.vertex_id(dart_id);
                let _ = cmap.replace_vertex(vid, *v);
                dart_id = cmap.beta::<1>(dart_id);
            }
        }

        let mut d_boundary = cmap.beta::<1>(*start);
        while d_boundary != *end {
            cmap.set_attribute::<Boundary>(d_boundary, Boundary::Left);
            cmap.set_attribute::<Boundary>(cmap.beta::<2>(d_boundary), Boundary::Right);
            d_boundary = cmap.beta::<1>(d_boundary);
        }
    }
}
