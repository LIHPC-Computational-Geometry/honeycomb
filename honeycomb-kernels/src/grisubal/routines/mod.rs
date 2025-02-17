//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

mod clip;
mod compute_intersecs;
mod compute_new_edges;
mod insert_intersecs;
mod insert_new_edges;
mod pre_processing;
mod process_intersecs_data;

// step 0
pub(crate) use pre_processing::*;

// step 1
pub(crate) use compute_intersecs::*;

// step 2
pub(crate) use process_intersecs_data::*;

// step 3
pub(crate) use insert_intersecs::*;

// step 4
pub(crate) use compute_new_edges::*;

// step 5
pub(crate) use insert_new_edges::*;

// optional clipping routines
pub(crate) use clip::{clip_left, clip_right};

use std::collections::HashMap;

use honeycomb_core::cmap::{DartIdType, EdgeIdType};

use crate::grisubal::model::GeometryVertex;

pub type Segments = HashMap<GeometryVertex, GeometryVertex>;

pub type IntersectionsPerEdge<T> = HashMap<EdgeIdType, Vec<(usize, T, DartIdType)>>;

pub type DartSlices = Vec<Vec<DartIdType>>;
