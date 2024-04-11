//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{AttributeBind, AttributeUpdate};
use num::ToPrimitive;

// ------ CONTENT

pub struct AttrSparseVec<T: AttributeBind + AttributeUpdate> {
    data: Vec<Option<T>>,
}

impl<T: AttributeBind + AttributeUpdate> AttrSparseVec<T> {
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

    pub fn insert(&mut self, index: T::IdentifierType, val: T) {
        let tmp = &mut self.data[index.to_usize().unwrap()];
        assert!(tmp.is_none());
        *tmp = Some(val);
    }

    pub fn replace(&mut self, index: T::IdentifierType, val: T) {
        let tmp = &mut self.data[index.to_usize().unwrap()];
        assert!(tmp.is_some());
        *tmp = Some(val);
    }

    pub fn remove(&mut self, index: T::IdentifierType) -> Option<T> {
        self.data.push(None);
        self.data.swap_remove(index.to_usize().unwrap())
    }
}

pub struct AttrCompactVec<T: AttributeBind + AttributeUpdate> {
    unused_data_slots: Vec<usize>,
    index_map: Vec<Option<usize>>,
    data: Vec<T>,
}

impl<T: AttributeBind + AttributeUpdate + Default> AttrCompactVec<T> {
    pub fn new(n_ids: usize, n_attributes: usize) -> Self {
        Self {
            unused_data_slots: (0..n_attributes).collect(),
            index_map: vec![None; n_ids],
            data: Vec::with_capacity(n_attributes),
        }
    }

    pub fn get(&self, index: T::IdentifierType) -> Option<&T> {
        self.index_map[index.to_usize().unwrap()].map(|idx| &self.data[idx])
    }

    pub fn get_mut(&mut self, index: T::IdentifierType) -> Option<&mut T> {
        self.index_map[index.to_usize().unwrap()].map(|idx| &mut self.data[idx])
    }

    pub fn insert(&mut self, index: T::IdentifierType, val: T) {
        let idx = &mut self.index_map[index.to_usize().unwrap()];
        assert!(idx.is_none());
        *idx = if let Some(unused_idx) = self.unused_data_slots.pop() {
            self.data[unused_idx] = val;
            Some(unused_idx)
        } else {
            self.data.push(val);
            Some(self.data.len())
        };
    }

    pub fn replace(&mut self, index: T::IdentifierType, val: T) {
        let idx = &self.index_map[index.to_usize().unwrap()];
        assert!(idx.is_some());
        self.data[idx.unwrap()] = val;
    }

    pub fn remove(&mut self, index: T::IdentifierType) -> Option<T> {
        self.index_map.push(None);
        if let Some(tmp) = self.index_map.swap_remove(index.to_usize().unwrap()) {
            self.unused_data_slots.push(tmp);
            self.data.push(T::default());
            return Some(self.data.swap_remove(tmp));
        };
        None
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
