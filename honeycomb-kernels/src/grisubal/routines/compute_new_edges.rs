//! Step 4 implementation
//!
//! Rebuild information about the edge that will be inserted into the map. This is done by using
//! the list of "atomic" segments to search for connections between intersections, discarding
//! regular points and registering points of interests.

// ------ IMPORTS

use super::Segments;
use crate::grisubal::model::{Geometry2, GeometryVertex, MapEdge};
use honeycomb_core::prelude::{CMap2, CoordsFloat, DartIdentifier};

// ------ CONTENT

pub(crate) fn generate_edge_data<T: CoordsFloat>(
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
