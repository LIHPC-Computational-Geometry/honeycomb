//! Attribute storage structures
//!
//! This module contains all code used to describe custom collections used to store attributes
//! (see [`AttributeBind`], [`AttributeUpdate`]).

// ------ IMPORTS

use crate::{AttributeBind, AttributeUpdate};
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
///
#[cfg_attr(feature = "utils", derive(Clone))]
pub struct AttrSparseVec<T: AttributeBind + AttributeUpdate> {
    /// Inner storage.
    data: Vec<Option<T>>,
}

impl<T: AttributeBind + AttributeUpdate> AttrSparseVec<T> {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// - `n_ids: usize` -- Upper bound of IDs used to index the attribute's values (in practice,
    /// the number of darts).
    ///
    /// # Return
    ///
    /// Return a [`AttrSparseVec`] object full of `None`.
    ///
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub fn new(n_ids: usize) -> Self {
        Self {
            data: (0..n_ids).map(|_| None).collect(),
        }
    }

    /// Extend the inner vector's length
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- number of `None` instances to append to the current storage.
    ///
    pub fn extend(&mut self, length: usize) {
        self.data.extend((0..length).map(|_| None));
    }

    /// Return the number of stored attributes (i.e. number of `Some(_)` instances)
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_attributes(&self) -> usize {
        self.data.iter().filter(|val| val.is_some()).count()
    }

    /// Getter
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return a reference to the value indexed by `index`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn get(&self, index: &T::IdentifierType) -> &Option<T> {
        &self.data[index.to_usize().unwrap()]
    }

    /// Setter
    ///
    /// Set the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn set(&mut self, index: &T::IdentifierType, val: T) {
        self.data[index.to_usize().unwrap()] = Some(val);
    }

    /// Setter
    ///
    /// Insert a value at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - **there is already a value associated to the specified index**
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn insert(&mut self, index: &T::IdentifierType, val: T) {
        let tmp = &mut self.data[index.to_usize().unwrap()];
        assert!(tmp.is_none());
        *tmp = Some(val);
    }

    /// Setter
    ///
    /// Replace the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Return
    ///
    /// Return an option containing the old value if it existed.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn replace(&mut self, index: &T::IdentifierType, val: T) -> Option<T> {
        self.data.push(Some(val));
        self.data.swap_remove(index.to_usize().unwrap())
    }

    /// Remove an item from the storage and return it
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return the item associated to the specified index. Note that the method will not panic if
    /// there was not one, it will simply return `None`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn remove(&mut self, index: &T::IdentifierType) -> Option<T> {
        self.data.push(None);
        self.data.swap_remove(index.to_usize().unwrap())
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
/// - a vector of `T` items, indexed by values of the first vector
///
/// This implementation should favor locality of reference over access logic.
///
/// # Generics
///
/// - `T: AttributeBind + AttributeUpdate + Clone` -- Type of the stored attributes. The
/// `Clone` implementation is required in order to return copied values & invalidate internal
/// storage slot.
///
/// # Example
///
/// **Currently, this type is not meant to be used directly** when operating on combinatorial maps,
/// but it is kept public because it should eventually be part of the map building system where
/// the user will add its own attributes and choose how they are stored. As such, no example
/// is provided.
///
#[cfg_attr(feature = "utils", derive(Clone))]
pub struct AttrCompactVec<T: AttributeBind + AttributeUpdate + Clone> {
    /// Tracker of unused internal slots.
    unused_data_slots: Vec<usize>,
    /// Map between attribute index and internal index.
    index_map: Vec<Option<usize>>,
    /// Inner storage.
    data: Vec<T>,
}

impl<T: AttributeBind + AttributeUpdate + Clone> AttrCompactVec<T> {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// - `n_ids: usize` -- Upper bound of IDs used to index the attribute's values (in practice,
    /// the number of darts).
    ///
    /// # Return
    ///
    /// Return a "value-empty" [`AttrSparseVec`] object.
    ///
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub fn new(n_ids: usize) -> Self {
        Self {
            unused_data_slots: Vec::new(),
            index_map: vec![None; n_ids],
            data: Vec::new(),
        }
    }

    /// Extend the inner vector's length
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- number of `None` instances to append to the current storage.
    ///
    pub fn extend(&mut self, length: usize) {
        self.index_map.extend((0..length).map(|_| None));
    }

    /// Return the number of stored attributes in the internal storage.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_attributes(&self) -> usize {
        self.data.len()
    }

    /// Return the number of stored, used attributes in the internal storage.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_used_attributes(&self) -> usize {
        self.data.len() - self.unused_data_slots.len()
    }

    /// Getter
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return an `Option` that may contain a reference to the value associated to `index`, if
    /// it exists.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn get(&self, index: &T::IdentifierType) -> Option<&T> {
        self.index_map[index.to_usize().unwrap()].map(|idx| &self.data[idx])
    }

    /// Setter
    ///
    /// Set the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn set(&mut self, index: &T::IdentifierType, val: T) {
        if let Some(idx) = self.index_map[index.to_usize().unwrap()] {
            // internal index is defined => there should be associated data
            self.data[idx] = val;
        } else if let Some(unused_idx) = self.unused_data_slots.pop() {
            // internal index is undefined => a) there is an unused internal slot
            self.data[unused_idx] = val;
            self.index_map[index.to_usize().unwrap()] = Some(unused_idx);
        } else {
            // internal index is undefined => b) there is no unused internal slot
            self.data.push(val);
            self.index_map[index.to_usize().unwrap()] = Some(self.data.len() - 1);
        }
    }

    /// Setter
    ///
    /// Insert a value at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - **there is already a value associated to the specified index**
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn insert(&mut self, index: &T::IdentifierType, val: T) {
        let idx = &mut self.index_map[index.to_usize().unwrap()];
        assert!(idx.is_none());
        *idx = if let Some(unused_idx) = self.unused_data_slots.pop() {
            self.data[unused_idx] = val;
            Some(unused_idx)
        } else {
            self.data.push(val);
            Some(self.data.len() - 1)
        };
    }

    /// Setter
    ///
    /// Replace the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Return
    ///
    /// Return an option containing the old value if it existed.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn replace(&mut self, index: &T::IdentifierType, val: T) -> Option<T> {
        let idx = &self.index_map[index.to_usize().unwrap()];
        assert!(idx.is_some());
        self.data.push(val);
        Some(self.data.swap_remove(idx.unwrap()))
    }

    /// Remove an item from the storage and return it
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return the item associated to the specified index. Note that the method will not panic if
    /// there was not one, it will simply return `None`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn remove(&mut self, index: &T::IdentifierType) -> Option<T> {
        self.index_map.push(None);
        if let Some(tmp) = self.index_map.swap_remove(index.to_usize().unwrap()) {
            self.unused_data_slots.push(tmp);
            return Some(self.data[tmp].clone());
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
