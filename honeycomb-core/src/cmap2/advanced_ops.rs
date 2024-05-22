//! Advanced operations implementation
//!
//! This module contains code used to implement advanced operations, e.g. some non-standard,
//! higher-level abstractions that are useful in meshing algorithms.

// ------ IMPORTS

use crate::{CMap2, CoordsFloat, EdgeIdentifier};

// ------ CONTENT

impl<T: CoordsFloat> CMap2<T> {
    pub fn split_edge(&mut self, edge_id: EdgeIdentifier) {
        todo!()
    }
}
