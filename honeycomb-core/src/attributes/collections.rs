//! Attribute storage structures
//!
//! This module contains all code used to describe custom collections used to store attributes
//! (see [`AttributeBind`], [`AttributeUpdate`]).

// ------ IMPORTS

use super::{AttributeBind, AttributeStorage, AttributeUpdate, UnknownAttributeStorage};
use crate::prelude::DartIdentifier;
use num_traits::ToPrimitive;
use stm::{atomically, StmError, TVar, Transaction};

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
/// **This type is not meant to be used directly** but used along the [`AttributeBind`] trait.
#[derive(Debug)]
pub struct AttrSparseVec<T: AttributeBind + AttributeUpdate + Copy> {
    /// Inner storage.
    data: Vec<TVar<Option<T>>>,
}

#[doc(hidden)]
impl<A: AttributeBind + AttributeUpdate + Copy> AttrSparseVec<A> {
    pub(crate) fn merge_core(
        &self,
        trans: &mut Transaction,
        out: DartIdentifier,
        lhs_inp: DartIdentifier,
        rhs_inp: DartIdentifier,
    ) -> Result<(), StmError> {
        let new_v = match (
            self.data[lhs_inp as usize].read(trans)?,
            self.data[rhs_inp as usize].read(trans)?,
        ) {
            (Some(v1), Some(v2)) => Some(AttributeUpdate::merge(v1, v2)),
            (Some(v), None) | (None, Some(v)) => Some(AttributeUpdate::merge_incomplete(v)),
            (None, None) => AttributeUpdate::merge_from_none(),
        };
        if new_v.is_none() {
            eprintln!("W: cannot merge two null attribute value");
            eprintln!("   setting new target value to `None`");
        }
        self.data[rhs_inp as usize].write(trans, None)?;
        self.data[lhs_inp as usize].write(trans, None)?;
        self.data[out as usize].write(trans, new_v)?;
        Ok(())
    }

    pub(crate) fn split_core(
        &self,
        trans: &mut Transaction,
        lhs_out: DartIdentifier,
        rhs_out: DartIdentifier,
        inp: DartIdentifier,
    ) -> Result<(), StmError> {
        if let Some(val) = self.data[inp as usize].read(trans)? {
            let (lhs_val, rhs_val) = AttributeUpdate::split(val);
            self.data[inp as usize].write(trans, None)?;
            self.data[lhs_out as usize].write(trans, Some(lhs_val))?;
            self.data[rhs_out as usize].write(trans, Some(rhs_val))?;
        } else {
            eprintln!("W: cannot split attribute value (not found in storage)");
            eprintln!("   setting both new values to `None`");
            self.data[lhs_out as usize].write(trans, None)?;
            self.data[rhs_out as usize].write(trans, None)?;
            //self.data[inp as usize].store(None, Ordering::Release);
        }
        Ok(())
    }

    pub(crate) fn set_core(
        &self,
        trans: &mut Transaction,
        id: &A::IdentifierType,
        val: A,
    ) -> Result<(), StmError> {
        self.data[id.to_usize().unwrap()].write(trans, Some(val))?;
        Ok(())
    }

    pub(crate) fn insert_core(
        &self,
        trans: &mut Transaction,
        id: &A::IdentifierType,
        val: A,
    ) -> Result<(), StmError> {
        let tmp = self.data[id.to_usize().unwrap()].replace(trans, Some(val))?;
        // assertion prevents the transaction from being validated, so the
        // storage will be left unchanged before the crash
        assert!(tmp.is_none());
        Ok(())
    }

    pub(crate) fn get_core(
        &self,
        trans: &mut Transaction,
        id: &A::IdentifierType,
    ) -> Result<Option<A>, StmError> {
        self.data[id.to_usize().unwrap()].read(trans)
    }

    pub(crate) fn replace_core(
        &self,
        trans: &mut Transaction,
        id: &A::IdentifierType,
        val: A,
    ) -> Result<Option<A>, StmError> {
        self.data[id.to_usize().unwrap()].replace(trans, Some(val))
    }

    pub(crate) fn remove_core(
        &self,
        trans: &mut Transaction,
        id: &A::IdentifierType,
    ) -> Result<Option<A>, StmError> {
        self.data[id.to_usize().unwrap()].replace(trans, None)
    }
}

unsafe impl<A: AttributeBind + AttributeUpdate + Copy> Send for AttrSparseVec<A> {}
unsafe impl<A: AttributeBind + AttributeUpdate + Copy> Sync for AttrSparseVec<A> {}

impl<A: AttributeBind + AttributeUpdate + Copy> UnknownAttributeStorage for AttrSparseVec<A> {
    fn new(length: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            data: (0..length).map(|_| TVar::new(None)).collect(),
        }
    }

    fn extend(&mut self, length: usize) {
        self.data.extend((0..length).map(|_| TVar::new(None)));
    }

    fn n_attributes(&self) -> usize {
        self.data
            .iter()
            .filter(|v| v.read_atomic().is_some())
            .count()
    }

    fn merge(&self, out: DartIdentifier, lhs_inp: DartIdentifier, rhs_inp: DartIdentifier) {
        atomically(|trans| self.merge_core(trans, out, lhs_inp, rhs_inp));
    }

    fn merge_transac(
        &self,
        trans: &mut Transaction,
        out: DartIdentifier,
        lhs_inp: DartIdentifier,
        rhs_inp: DartIdentifier,
    ) -> Result<(), StmError> {
        self.merge_core(trans, out, lhs_inp, rhs_inp)
    }

    fn split(&self, lhs_out: DartIdentifier, rhs_out: DartIdentifier, inp: DartIdentifier) {
        atomically(|trans| self.split_core(trans, lhs_out, rhs_out, inp));
    }

    fn split_transac(
        &self,
        trans: &mut Transaction,
        lhs_out: DartIdentifier,
        rhs_out: DartIdentifier,
        inp: DartIdentifier,
    ) -> Result<(), StmError> {
        self.split_core(trans, lhs_out, rhs_out, inp)
    }
}

impl<A: AttributeBind + AttributeUpdate + Copy> AttributeStorage<A> for AttrSparseVec<A> {
    fn set(&self, id: A::IdentifierType, val: A) {
        atomically(|trans| self.set_core(trans, &id, val));
    }

    fn set_transac(
        &self,
        trans: &mut Transaction,
        id: <A as AttributeBind>::IdentifierType,
        val: A,
    ) -> Result<(), StmError> {
        self.set_core(trans, &id, val)
    }

    fn insert(&self, id: A::IdentifierType, val: A) {
        atomically(|trans| self.insert_core(trans, &id, val));
    }

    fn insert_transac(
        &self,
        trans: &mut Transaction,
        id: <A as AttributeBind>::IdentifierType,
        val: A,
    ) -> Result<(), StmError> {
        self.insert_core(trans, &id, val)
    }

    fn get(&self, id: A::IdentifierType) -> Option<A> {
        atomically(|trans| self.get_core(trans, &id))
    }

    fn get_transac(
        &self,
        trans: &mut Transaction,
        id: <A as AttributeBind>::IdentifierType,
    ) -> Result<Option<A>, StmError> {
        self.get_core(trans, &id)
    }

    fn replace(&self, id: A::IdentifierType, val: A) -> Option<A> {
        atomically(|trans| self.replace_core(trans, &id, val))
    }

    fn replace_transac(
        &self,
        trans: &mut Transaction,
        id: <A as AttributeBind>::IdentifierType,
        val: A,
    ) -> Result<Option<A>, StmError> {
        self.replace_core(trans, &id, val)
    }

    fn remove(&self, id: A::IdentifierType) -> Option<A> {
        atomically(|trans| self.remove_core(trans, &id))
    }

    fn remove_transac(
        &self,
        trans: &mut Transaction,
        id: <A as AttributeBind>::IdentifierType,
    ) -> Result<Option<A>, StmError> {
        self.remove_core(trans, &id)
    }
}

#[cfg(feature = "utils")]
impl<T: AttributeBind + AttributeUpdate + Copy> AttrSparseVec<T> {
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
        self.n_attributes() * size_of::<TVar<Option<T>>>()
    }
}

/*
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
/// **This type is not meant to be used directly** but used along the [`AttributeBind`] trait.
#[derive(Debug)]
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
        match (self.remove(lhs_inp.into()), self.remove(rhs_inp.into())) {
            (Some(v1), Some(v2)) => self.set(out.into(), AttributeUpdate::merge(v1, v2)),
            (Some(v), None) | (None, Some(v)) => {
                self.set(out.into(), AttributeUpdate::merge_incomplete(v));
            }
            (None, None) => {
                if let Some(v) = AttributeUpdate::merge_from_none() {
                    self.set(out.into(), v);
                } else {
                    eprintln!("W: cannot merge two null attribute value");
                    eprintln!("   setting new target value to `None`");
                    let _ = self.remove(out.into());
                }
            }
        };
    }

    fn split(&mut self, lhs_out: DartIdentifier, rhs_out: DartIdentifier, inp: DartIdentifier) {
        let new_val = self.remove(inp.into());
        if let Some(val) = new_val {
            let (lhs_val, rhs_val) = AttributeUpdate::split(val);
            self.set(lhs_out.into(), lhs_val);
            self.set(rhs_out.into(), rhs_val);
        } else {
            eprintln!("W: cannot split attribute value (not found in storage)");
            eprintln!("   setting both new values to `None`");
            let _ = self.remove(lhs_out.into());
            let _ = self.remove(rhs_out.into());
        }
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

    fn replace(&mut self, id: A::IdentifierType, val: A) -> Option<A> {
        let idx = &self.index_map[id.to_usize().unwrap()];
        if idx.is_none() {
            return None;
        }
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
 */
