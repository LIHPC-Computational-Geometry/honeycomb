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

impl<T: AttributeBind + AttributeUpdate> Index<T::IdentifierType> for AttributeSparseVec<T> {
    type Output = Option<T>;

    fn index(&self, index: T::IdentifierType) -> &Self::Output {
        &self.data[index.into()]
    }
}

impl<T: AttributeBind + AttributeUpdate> IndexMut<T::IdentifierType> for AttributeSparseVec<T> {
    fn index_mut(&mut self, index: T::IdentifierType) -> &mut Self::Output {
        &mut self.data[index.into()]
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

impl<T: AttributeBind + AttributeUpdate> Index<T::IdentifierType> for AttributeCompactVec<T> {
    type Output = Option<T>;

    fn index(&self, index: T::IdentifierType) -> &Self::Output {
        &self.index_map[index.into()].map(|id| self.data[id])
    }
}

impl<T: AttributeBind + AttributeUpdate> IndexMut<T::IdentifierType> for AttributeCompactVec<T> {
    fn index_mut(&mut self, index: T::IdentifierType) -> &mut Self::Output {
        &mut self.index_map[index.into()].map(|id| self.data[id])
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
