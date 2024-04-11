//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{AttributeBind, AttributeUpdate};
use num::ToPrimitive;

// ------ CONTENT

/// Custom storage structure for [attributes]
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
/// ```text
///
/// ```
pub struct AttrSparseVec<T: AttributeBind + AttributeUpdate> {
    data: Vec<Option<T>>,
}

impl<T: AttributeBind + AttributeUpdate> AttrSparseVec<T> {
    pub fn new(n_ids: usize) -> Self {
        Self {
            data: (0..n_ids).map(|_| None).collect(),
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

/// Custom storage structure for [attributes]
///
/// This structured is used to store user-defined attributes using two internal collections:
/// - a vector of `Option<usize>`, effectively acting as a map from identifiers to internal indices
/// - a vector of `T` items, indexed by values of the first vector
///
/// This implementation should favor locality of reference over access logic.
///
/// # Generics
///
/// - `T: AttributeBind + AttributeUpdate + Default` -- Type of the stored attributes. The
/// `Default` implementation is required in order to create dummy values for unused slots.
///
/// # Example
///
/// ```text
///
/// ```
pub struct AttrCompactVec<T: AttributeBind + AttributeUpdate + Default> {
    unused_data_slots: Vec<usize>,
    index_map: Vec<Option<usize>>,
    data: Vec<T>,
}

impl<T: AttributeBind + AttributeUpdate + Default> AttrCompactVec<T> {
    pub fn new(n_ids: usize, n_attributes: usize) -> Self {
        Self {
            unused_data_slots: (0..n_attributes).collect(),
            index_map: vec![None; n_ids],
            data: (0..n_attributes).map(|_| T::default()).collect(),
        }
    }

    pub fn get(&self, index: T::IdentifierType) -> Option<&T> {
        self.index_map[index.to_usize().unwrap()].map(|idx| &self.data[idx])
    }

    pub fn get_mut(&mut self, index: T::IdentifierType) -> Option<&mut T> {
        self.index_map[index.to_usize().unwrap()].map(|idx| &mut self.data[idx])
    }

    pub fn set(&mut self, index: T::IdentifierType, val: T) {
        if let Some(idx) = self.index_map[index.to_usize().unwrap()] {
            self.data[idx] = val;
        } else if let Some(unused_idx) = self.unused_data_slots.pop() {
            self.data[unused_idx] = val;
            self.index_map[index.to_usize().unwrap()] = Some(unused_idx);
        } else {
            self.data.push(val);
            self.index_map[index.to_usize().unwrap()] = Some(self.data.len());
        }
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
    use super::*;
    use crate::{FaceIdentifier, OrbitPolicy};

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Temperature {
        pub val: f32,
    }

    impl AttributeUpdate for Temperature {
        fn merge(attr1: Self, attr2: Self) -> Self {
            Temperature {
                val: (attr1.val + attr2.val) / 2.0,
            }
        }

        fn split(attr: Self) -> (Self, Self) {
            (attr, attr)
        }

        fn merge_undefined(attr: Option<Self>) -> Self {
            attr.unwrap_or(Temperature { val: 0.0 })
        }
    }

    impl AttributeBind for Temperature {
        type IdentifierType = FaceIdentifier;
        fn binds_to<'a>() -> OrbitPolicy<'a> {
            OrbitPolicy::Face
        }
    }

    impl From<f32> for Temperature {
        fn from(val: f32) -> Self {
            Self { val }
        }
    }

    macro_rules! generate_sparse {
        ($name: ident) => {
            let mut $name = AttrSparseVec::<Temperature>::new(10);
            $name.insert(0, Temperature::from(273.0));
            $name.insert(1, Temperature::from(275.0));
            $name.insert(2, Temperature::from(277.0));
            $name.insert(3, Temperature::from(279.0));
            $name.insert(4, Temperature::from(281.0));
            $name.insert(5, Temperature::from(283.0));
            $name.insert(6, Temperature::from(285.0));
            $name.insert(7, Temperature::from(287.0));
            $name.insert(8, Temperature::from(289.0));
            $name.insert(9, Temperature::from(291.0));
        };
    }

    #[test]
    fn sparse_vec_get_set_get() {
        generate_sparse!(storage);
        assert_eq!(storage.get(3), &Some(Temperature::from(279.0)));
        storage.set(3, Temperature::from(280.0));
        assert_eq!(storage.get(3), &Some(Temperature::from(280.0)));
    }

    #[test]
    fn sparse_vec_get_replace_get() {
        generate_sparse!(storage);
        assert_eq!(storage.get(3), &Some(Temperature::from(279.0)));
        storage.replace(3, Temperature::from(280.0));
        assert_eq!(storage.get(3), &Some(Temperature::from(280.0)));
    }

    #[test]
    #[should_panic]
    fn sparse_vec_get_insert_get() {
        generate_sparse!(storage);
        assert_eq!(storage.get(3), &Some(Temperature::from(279.0)));
        storage.insert(3, Temperature::from(280.0)); // panic
    }

    #[test]
    fn sparse_vec_remove() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    }

    #[test]
    fn sparse_vec_remove_remove() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        assert!(storage.remove(3).is_none());
    }

    #[test]
    fn sparse_vec_remove_get() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        assert!(storage.get(3).is_none());
    }

    #[test]
    fn sparse_vec_remove_set() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.set(3, Temperature::from(280.0));
        assert!(storage.get(3).is_some());
    }

    #[test]
    fn sparse_vec_remove_insert() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.insert(3, Temperature::from(280.0));
        assert!(storage.get(3).is_some());
    }

    #[test]
    #[should_panic]
    fn sparse_vec_remove_replace() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.replace(3, Temperature::from(280.0)); // panic
    }

    macro_rules! generate_compact {
        ($name: ident) => {
            let mut $name = AttrCompactVec::<Temperature>::new(10, 10);
            $name.insert(0, Temperature::from(273.0));
            $name.insert(1, Temperature::from(275.0));
            $name.insert(2, Temperature::from(277.0));
            $name.insert(3, Temperature::from(279.0));
            $name.insert(4, Temperature::from(281.0));
            $name.insert(5, Temperature::from(283.0));
            $name.insert(6, Temperature::from(285.0));
            $name.insert(7, Temperature::from(287.0));
            $name.insert(8, Temperature::from(289.0));
            $name.insert(9, Temperature::from(291.0));
        };
    }

    #[test]
    fn compact_vec_get_set_get() {
        generate_compact!(storage);
        assert_eq!(storage.get(3), Some(&Temperature::from(279.0)));
        storage.set(3, Temperature::from(280.0));
        assert_eq!(storage.get(3), Some(&Temperature::from(280.0)));
    }

    #[test]
    fn compact_vec_get_replace_get() {
        generate_compact!(storage);
        assert_eq!(storage.get(3), Some(&Temperature::from(279.0)));
        storage.replace(3, Temperature::from(280.0));
        assert_eq!(storage.get(3), Some(&Temperature::from(280.0)));
    }

    #[test]
    #[should_panic]
    fn compact_vec_get_insert_get() {
        generate_compact!(storage);
        assert_eq!(storage.get(3), Some(&Temperature::from(279.0)));
        storage.insert(3, Temperature::from(280.0)); // panic
    }

    #[test]
    fn compact_vec_remove() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    }

    #[test]
    fn compact_vec_remove_remove() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        assert!(storage.remove(3).is_none());
    }

    #[test]
    fn compact_vec_remove_get() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        assert!(storage.get(3).is_none());
    }

    #[test]
    fn compact_vec_remove_set() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.set(3, Temperature::from(280.0));
        assert!(storage.get(3).is_some());
    }

    #[test]
    fn compact_vec_remove_insert() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.insert(3, Temperature::from(280.0));
        assert!(storage.get(3).is_some());
    }

    #[test]
    #[should_panic]
    fn compact_vec_remove_replace() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.replace(3, Temperature::from(280.0)); // panic
    }
}
