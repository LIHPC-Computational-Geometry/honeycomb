//! Attribute storage structures
//!
//! This module contains all code used to describe custom collections used to store attributes
//! (see [`AttributeBind`], [`AttributeUpdate`]).

// ------ IMPORTS

use super::{AttributeBind, AttributeStorage, AttributeUpdate, UnknownAttributeStorage};
use crate::prelude::DartIdentifier;
use num::ToPrimitive;

// ------ CONTENT

/// Custom storage structure for attributes
///
/// This structured is used to store user-defined attributes using a vector of `Option<T>` items.
/// This means that valid attributes value may be separated by an arbitrary number of `None`.
///
/// This implementation should favor access logic over locality of reference.
///
/// # Generics
///
/// - `T: AttributeBind + AttributeUpdate` -- Type of the stored attributes.
///
/// # Example
///
/// **Currently, this type is not meant to be used directly** when operating on combinatorial maps,
/// but it is kept public because it should eventually be part of the map building system where
/// the user will add its own attributes and choose how they are stored. As such, no example
/// is provided.
#[derive(Debug)]
#[cfg_attr(feature = "utils", derive(Clone))]
pub struct AttrSparseVec<T: AttributeBind + AttributeUpdate> {
    /// Inner storage.
    data: Vec<Option<T>>,
}

impl<A: AttributeBind + AttributeUpdate + Copy> UnknownAttributeStorage for AttrSparseVec<A> {
    fn new(length: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            data: (0..length).map(|_| None).collect(),
        }
    }

    fn extend(&mut self, length: usize) {
        self.data.extend((0..length).map(|_| None));
    }

    fn n_attributes(&self) -> usize {
        self.data.iter().filter(|val| val.is_some()).count()
    }

    fn merge(&mut self, out: DartIdentifier, lhs_inp: DartIdentifier, rhs_inp: DartIdentifier) {
        let new_val = match (self.remove(lhs_inp.into()), self.remove(rhs_inp.into())) {
            (Some(v1), Some(v2)) => AttributeUpdate::merge(v1, v2),
            (Some(v), None) | (None, Some(v)) => AttributeUpdate::merge_undefined(Some(v)),
            (None, None) => AttributeUpdate::merge_undefined(None),
        };
        self.set(out.into(), new_val);
    }

    fn split(&mut self, lhs_out: DartIdentifier, rhs_out: DartIdentifier, inp: DartIdentifier) {
        let new_val = self
            .remove(inp.into())
            .expect("E: cannot split attribute value - value not found in storage");
        let (lhs_val, rhs_val) = AttributeUpdate::split(new_val);
        self.set(lhs_out.into(), lhs_val);
        self.set(rhs_out.into(), rhs_val);
    }
}

impl<A: AttributeBind + AttributeUpdate + Copy> AttributeStorage<A> for AttrSparseVec<A> {
    fn set(&mut self, id: A::IdentifierType, val: A) {
        self.data[id.to_usize().unwrap()] = Some(val);
    }

    fn insert(&mut self, id: A::IdentifierType, val: A) {
        let tmp = &mut self.data[id.to_usize().unwrap()];
        assert!(tmp.is_none());
        *tmp = Some(val);
    }

    fn get(&self, id: A::IdentifierType) -> Option<A> {
        self.data[id.to_usize().unwrap()]
    }

    fn replace(&mut self, id: A::IdentifierType, val: A) -> Option<A> {
        self.data.push(Some(val));
        self.data.swap_remove(id.to_usize().unwrap())
    }

    fn remove(&mut self, id: A::IdentifierType) -> Option<A> {
        self.data.push(None);
        self.data.swap_remove(id.to_usize().unwrap())
    }
}

#[cfg(feature = "utils")]
impl<T: AttributeBind + AttributeUpdate + Clone> AttrSparseVec<T> {
    /// Return the amount of space allocated for the storage.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn allocated_size(&self) -> usize {
        self.data.capacity() * std::mem::size_of::<Option<T>>()
    }

    /// Return the total amount of space used by the storage.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn effective_size(&self) -> usize {
        self.data.len() * std::mem::size_of::<Option<T>>()
    }

    /// Return the amount of space used by valid entries of the storage.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn used_size(&self) -> usize {
        self.data.iter().filter(|val| val.is_some()).count() * std::mem::size_of::<Option<T>>()
    }
}

/// Custom storage structure for attributes
///
/// This structured is used to store user-defined attributes using two internal collections:
/// - a vector of `Option<usize>`, effectively acting as a map from identifiers to internal indices
/// - a vector of `A` items, indexed by values of the first vector
///
/// This implementation should favor locality of reference over access logic.
///
/// # Generics
///
/// - `A: AttributeBind + AttributeUpdate + Clone` -- Type of the stored attributes. The
///   `Clone` implementation is required in order to return copied values & invalidate internal
///   storage slot.
///
/// # Example
///
/// **Currently, this type is not meant to be used directly** when operating on combinatorial maps,
/// but it is kept public because it should eventually be part of the map building system where
/// the user will add its own attributes and choose how they are stored. As such, no example
/// is provided.
///
#[derive(Debug)]
#[cfg_attr(feature = "utils", derive(Clone))]
pub struct AttrCompactVec<A: AttributeBind + AttributeUpdate + Clone> {
    /// Tracker of unused internal slots.
    unused_data_slots: Vec<usize>,
    /// Map between attribute index and internal index.
    index_map: Vec<Option<usize>>,
    /// Inner storage.
    data: Vec<A>,
}

impl<A: AttributeBind + AttributeUpdate + Copy> UnknownAttributeStorage for AttrCompactVec<A> {
    fn new(length: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            unused_data_slots: Vec::new(),
            index_map: vec![None; length],
            data: Vec::new(),
        }
    }

    fn extend(&mut self, length: usize) {
        self.index_map.extend((0..length).map(|_| None));
    }

    fn n_attributes(&self) -> usize {
        self.data.len() - self.unused_data_slots.len()
    }

    fn merge(&mut self, out: DartIdentifier, lhs_inp: DartIdentifier, rhs_inp: DartIdentifier) {
        let new_val = match (self.remove(lhs_inp.into()), self.remove(rhs_inp.into())) {
            (Some(v1), Some(v2)) => AttributeUpdate::merge(v1, v2),
            (Some(v), None) | (None, Some(v)) => AttributeUpdate::merge_undefined(Some(v)),
            (None, None) => AttributeUpdate::merge_undefined(None),
        };
        self.set(out.into(), new_val);
    }

    fn split(&mut self, lhs_out: DartIdentifier, rhs_out: DartIdentifier, inp: DartIdentifier) {
        let new_val = self
            .remove(inp.into())
            .expect("E: cannot split attribute value - value not found in storage");
        let (lhs_val, rhs_val) = AttributeUpdate::split(new_val);
        self.set(lhs_out.into(), lhs_val);
        self.set(rhs_out.into(), rhs_val);
    }
}

impl<A: AttributeBind + AttributeUpdate + Copy> AttributeStorage<A> for AttrCompactVec<A> {
    fn set(&mut self, id: A::IdentifierType, val: A) {
        if let Some(idx) = self.index_map[id.to_usize().unwrap()] {
            // internal index is defined => there should be associated data
            self.data[idx] = val;
        } else if let Some(unused_idx) = self.unused_data_slots.pop() {
            // internal index is undefined => a) there is an unused internal slot
            self.data[unused_idx] = val;
            self.index_map[id.to_usize().unwrap()] = Some(unused_idx);
        } else {
            // internal index is undefined => b) there is no unused internal slot
            self.data.push(val);
            self.index_map[id.to_usize().unwrap()] = Some(self.data.len() - 1);
        }
    }

    fn insert(&mut self, id: A::IdentifierType, val: A) {
        let idx = &mut self.index_map[id.to_usize().unwrap()];
        assert!(idx.is_none());
        *idx = if let Some(unused_idx) = self.unused_data_slots.pop() {
            self.data[unused_idx] = val;
            Some(unused_idx)
        } else {
            self.data.push(val);
            Some(self.data.len() - 1)
        };
    }

    fn get(&self, id: A::IdentifierType) -> Option<A> {
        self.index_map[id.to_usize().unwrap()].map(|idx| self.data[idx])
    }

    // FIXME: panics instead of returning None
    fn replace(&mut self, id: A::IdentifierType, val: A) -> Option<A> {
        let idx = &self.index_map[id.to_usize().unwrap()];
        assert!(idx.is_some());
        self.data.push(val);
        Some(self.data.swap_remove(idx.unwrap()))
    }

    fn remove(&mut self, id: A::IdentifierType) -> Option<A> {
        self.index_map.push(None);
        if let Some(tmp) = self.index_map.swap_remove(id.to_usize().unwrap()) {
            self.unused_data_slots.push(tmp);
            return Some(self.data[tmp]);
        };
        None
    }
}

#[cfg(feature = "utils")]
impl<T: AttributeBind + AttributeUpdate + Clone> AttrCompactVec<T> {
    /// Return the amount of space allocated for the storage.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn allocated_size(&self) -> usize {
        self.unused_data_slots.capacity() * std::mem::size_of::<usize>()
            + self.index_map.capacity() * std::mem::size_of::<Option<usize>>()
            + self.data.capacity() * std::mem::size_of::<T>()
    }

    /// Return the total amount of space used by the storage.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn effective_size(&self) -> usize {
        self.unused_data_slots.len() * std::mem::size_of::<usize>()
            + self.index_map.len() * std::mem::size_of::<Option<usize>>()
            + self.data.len() * std::mem::size_of::<T>()
    }

    /// Return the amount of space used by valid entries of the storage.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn used_size(&self) -> usize {
        self.unused_data_slots.len() * std::mem::size_of::<usize>()
            + self.index_map.iter().filter(|val| val.is_some()).count()
                * std::mem::size_of::<Option<usize>>()
            + self.data.len() * std::mem::size_of::<T>()
    }
}
