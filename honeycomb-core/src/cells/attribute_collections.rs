//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{AttributeBind, AttributeUpdate};
use std::ops::{Index, IndexMut};

// ------ CONTENT

pub struct AttributeSparseVec<T: AttributeBind + AttributeUpdate> {
    data: Vec<Option<T>>,
}

impl<T: AttributeBind + AttributeUpdate> AttributeSparseVec<T> {
    pub fn new(length: usize) -> Self {
        Self {
            data: (0..length).map(|_| None).collect(),
        }
    }
}

pub struct AttributeCompactVec<T: AttributeBind + AttributeUpdate> {
    index_map: Vec<Option<usize>>,
    data: Vec<T>,
}

impl<T: AttributeBind + AttributeUpdate> AttributeCompactVec<T> {
    pub fn new(n_ids: usize, n_attributes: usize) -> Self {
        Self {
            index_map: vec![None; n_ids],
            data: Vec::with_capacity(n_attributes),
        }
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
