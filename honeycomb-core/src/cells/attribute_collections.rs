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
    inner: Vec<Option<T>>,
}

impl<T: AttributeBind + AttributeUpdate> AttributeSparseVec<T> {
    pub fn new(length: usize) -> Self {
        Self {
            inner: (0..length).map(|_| None).collect(),
        }
    }
}

impl<T: AttributeBind + AttributeUpdate> Index<T::IdentifierType> for AttributeSparseVec<T> {
    type Output = Option<T>;

    fn index(&self, index: T::IdentifierType) -> &Self::Output {
        &self.inner[index.into()]
    }
}

impl<T: AttributeBind + AttributeUpdate> IndexMut<T::IdentifierType> for AttributeSparseVec<T> {
    fn index_mut(&mut self, index: T::IdentifierType) -> &mut Self::Output {
        &mut self.inner[index.into()]
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
