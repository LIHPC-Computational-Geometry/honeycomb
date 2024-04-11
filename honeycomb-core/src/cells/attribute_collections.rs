//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{AttributeBind, AttributeUpdate};
use num::ToPrimitive;

// ------ CONTENT

pub struct AttributeSparseVec<T: AttributeBind + AttributeUpdate> {
    data: Vec<Option<T>>,
}

impl<T: AttributeBind + AttributeUpdate> AttributeSparseVec<T> {
    pub fn new(n_attributes: usize) -> Self {
        Self {
            data: (0..n_attributes).map(|_| None).collect(),
        }
    }

    pub fn get(&self, index: T::IdentifierType) -> &Option<T> {
        &self.data[index.to_usize().unwrap()]
    }

    pub fn get_mut(&mut self, index: T::IdentifierType) -> &mut Option<T> {
        &mut self.data[index.to_usize().unwrap()]
    }

    pub fn set(&mut self, index: T::IdentifierType, val: T) {
        self.data[index.to_usize().unwrap()] = Some(val);
    }

    pub fn remove(&mut self, index: T::IdentifierType) -> Option<T> {
        self.data.push(None);
        self.data.swap_remove(index.to_usize().unwrap())
    }
}

pub struct AttributeCompactVec<T: AttributeBind + AttributeUpdate> {
    index_map: Vec<Option<usize>>,
    data: Vec<T>,
}

impl<T: AttributeBind + AttributeUpdate + Default> AttributeCompactVec<T> {
    pub fn new(n_ids: usize, n_attributes: usize) -> Self {
        Self {
            index_map: vec![None; n_ids],
            data: (0..n_attributes).map(|_| T::default()).collect(),
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
