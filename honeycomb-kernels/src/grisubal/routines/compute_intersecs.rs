//! Step 1 implementation
//!
//! The aim of this step is to build an exhaustive list of the segments making up
//! the geometry intersected with the grid: For each segment, if both vertices
//! do not belong to the same cell, we break it into sub-segments until it is the case.

use std::{
    cmp::{max, min},
    collections::VecDeque,
};

use honeycomb_core::cmap::{CMap2, DartIdType, NULL_DART_ID};
use honeycomb_core::geometry::{CoordsFloat, Vertex2};

use crate::grisubal::model::{Geometry2, GeometryVertex, GridCellId};

use super::Segments;

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

#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub(crate) fn generate_intersection_data<T: CoordsFloat>(
    cmap: &CMap2<T>,
    geometry: &Geometry2<T>,
    [nx, _ny]: [usize; 2],
    [cx, cy]: [T; 2],
    origin: Vertex2<T>,
) -> (Segments, Vec<(DartIdType, T)>) {
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
                GridCellId::l1_dist(&c1, &c2),
                GridCellId::offset(&c1, &c2),
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
    let prefix_sum = tmp
        .iter()
        .map(|(dist, _, _, _, _, _, _)| dist)
        .scan(0, |state, &dist| {
            *state += dist;
            Some(*state - dist) // we want an offset, not the actual sum
        });
    // preallocate the intersection vector
    let mut intersection_metadata = vec![(NULL_DART_ID, T::nan()); n_intersec];

    let new_segments: Segments = tmp.iter().zip(prefix_sum).flat_map(|(&(dist, diff, v1, v2, v1_id, v2_id, c1), start)| {
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
                let d_base = (1 + 4 * c1.0 + nx * 4 * c1.1) as DartIdType;
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
                    .read_vertex(cmap.vertex_id(dart_id))
                    .expect("E: found a topological vertex with no associated coordinates");
                // compute relative position of the intersection on the intersecting edges
                // `s` is relative to the segment `v1v2`, `t` to the grid's segment (the origin being `v_dart`)
                #[rustfmt::skip]
                let (_s, t) = match diff {
                    (-1,  0) => left_intersec!(v1, v2, v_dart, cy),
                    ( 1,  0) => right_intersec!(v1, v2, v_dart, cy),
                    ( 0, -1) => down_intersec!(v1, v2, v_dart, cx),
                    ( 0,  1) => up_intersec!(v1, v2, v_dart, cx),
                    _ => unreachable!(),
                };

                let id = start;
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
                let i_ids = start..start+dist;
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
                                    (1 + 4 * x + (nx * 4 * c1.1) as isize) as DartIdType;
                                // intersected dart
                                let dart_id = if i.is_positive() {
                                    d_base + 1
                                } else {
                                    d_base + 3
                                };
                                // vertex associated to the intersected dart
                                let v_dart = cmap.read_vertex(cmap.vertex_id(dart_id))
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
                                let d_base = (1 + 4 * c1.0 + nx * 4 * y as usize) as DartIdType;
                                // intersected dart
                                let dart_id = if j.is_positive() { d_base + 2 } else { d_base };
                                // vertex associated to the intersected dart
                                let v_dart = cmap.read_vertex(cmap.vertex_id(dart_id))
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

                        let mut intersec_data: Vec<(T, T, DartIdType)> = subgrid_cells
                            .map(|(x, y)| {
                                // cell base dart
                                let d_base = (1 + 4 * x + nx as isize * 4 * y) as DartIdType;
                                // (potentially) intersected darts
                                let vdart_id = if i.is_positive() {
                                    d_base + 1
                                } else {
                                    d_base + 3
                                };
                                let hdart_id = if j.is_positive() { d_base + 2 } else { d_base };
                                // associated vertices
                                let v_vdart = cmap.read_vertex(cmap.vertex_id(vdart_id))
                                    .expect("E: found a topological vertex with no associated coordinates");
                                let v_hdart = cmap.read_vertex(cmap.vertex_id(hdart_id))
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
