//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULES

mod step0;
mod step1;
mod step2;
mod step3;
mod step4;
mod step5;

// ------ RE-EXPORTS

pub(crate) use step0::*;
pub(crate) use step1::*;
pub(crate) use step2::*;
pub(crate) use step3::*;
pub(crate) use step4::*;
pub(crate) use step5::*;

// ------ IMPORTS

use crate::grisubal::model::GeometryVertex;
use honeycomb_core::prelude::{DartIdentifier, EdgeIdentifier};
use std::collections::HashMap;

// ------ CONTENT

pub type Segments = HashMap<GeometryVertex, GeometryVertex>;

pub type IntersectionsPerEdge<T> = HashMap<EdgeIdentifier, Vec<(usize, T, DartIdentifier)>>;

pub type DartSlices = Vec<Vec<DartIdentifier>>;
