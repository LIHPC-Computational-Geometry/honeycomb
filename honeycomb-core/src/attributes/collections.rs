//! Attribute storage structures
//!
//! This module contains all code used to describe custom collections used to store attributes
//! (see [`AttributeBind`], [`AttributeUpdate`]).

// ------ IMPORTS

use std::ops::Index;

use super::{AttributeBind, AttributeStorage, AttributeUpdate, UnknownAttributeStorage};
use crate::cmap::DartId;
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
pub struct AttrSparseVec<T: AttributeBind + AttributeUpdate> {
    /// Inner storage.
    data: Vec<TVar<Option<T>>>,
}

impl<A: AttributeBind + AttributeUpdate> Index<DartId> for AttrSparseVec<A> {
    type Output = TVar<Option<A>>;

    fn index(&self, index: DartId) -> &Self::Output {
        assert!((index.0 as usize) < self.data.len());
        &self.data[index.0 as usize]
    }
}

#[doc(hidden)]
impl<A: AttributeBind + AttributeUpdate> AttrSparseVec<A> {
    pub(crate) fn merge_core(
        &self,
        trans: &mut Transaction,
        out: DartId,
        lhs_inp: DartId,
        rhs_inp: DartId,
    ) -> Result<(), StmError> {
        let new_v = match (self[lhs_inp].read(trans)?, self[rhs_inp].read(trans)?) {
            (Some(v1), Some(v2)) => Some(AttributeUpdate::merge(v1, v2)),
            (Some(v), None) | (None, Some(v)) => Some(AttributeUpdate::merge_incomplete(v)),
            (None, None) => AttributeUpdate::merge_from_none(),
        };
        if new_v.is_none() {
            eprintln!("W: cannot merge two null attribute value");
            eprintln!("   setting new target value to `None`");
        }
        self[rhs_inp].write(trans, None)?;
        self[lhs_inp].write(trans, None)?;
        self[out].write(trans, new_v)?;
        Ok(())
    }

    pub(crate) fn split_core(
        &self,
        trans: &mut Transaction,
        lhs_out: DartId,
        rhs_out: DartId,
        inp: DartId,
    ) -> Result<(), StmError> {
        if let Some(val) = self[inp].read(trans)? {
            let (lhs_val, rhs_val) = AttributeUpdate::split(val);
            self[inp].write(trans, None)?;
            self[lhs_out].write(trans, Some(lhs_val))?;
            self[rhs_out].write(trans, Some(rhs_val))?;
        } else {
            eprintln!("W: cannot split attribute value (not found in storage)");
            eprintln!("   setting both new values to `None`");
            self[lhs_out].write(trans, None)?;
            self[rhs_out].write(trans, None)?;
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
        self[<A::IdentifierType as Into<DartId>>::into(id.clone())].write(trans, Some(val))?;
        Ok(())
    }

    pub(crate) fn insert_core(
        &self,
        trans: &mut Transaction,
        id: &A::IdentifierType,
        val: A,
    ) -> Result<(), StmError> {
        let tmp = self[<A::IdentifierType as Into<DartId>>::into(id.clone())]
            .replace(trans, Some(val))?;
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
        self[<A::IdentifierType as Into<DartId>>::into(id.clone())].read(trans)
    }

    pub(crate) fn replace_core(
        &self,
        trans: &mut Transaction,
        id: &A::IdentifierType,
        val: A,
    ) -> Result<Option<A>, StmError> {
        self[<A::IdentifierType as Into<DartId>>::into(id.clone())].replace(trans, Some(val))
    }

    pub(crate) fn remove_core(
        &self,
        trans: &mut Transaction,
        id: &A::IdentifierType,
    ) -> Result<Option<A>, StmError> {
        // maybe make a to_usize function ?
        self[<A::IdentifierType as Into<DartId>>::into(id.clone())].replace(trans, None)
    }
}

unsafe impl<A: AttributeBind + AttributeUpdate> Send for AttrSparseVec<A> {}
unsafe impl<A: AttributeBind + AttributeUpdate> Sync for AttrSparseVec<A> {}

impl<A: AttributeBind + AttributeUpdate> UnknownAttributeStorage for AttrSparseVec<A> {
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

    fn merge(&self, out: DartId, lhs_inp: DartId, rhs_inp: DartId) {
        atomically(|trans| self.merge_core(trans, out, lhs_inp, rhs_inp));
    }

    fn merge_transac(
        &self,
        trans: &mut Transaction,
        out: DartId,
        lhs_inp: DartId,
        rhs_inp: DartId,
    ) -> Result<(), StmError> {
        self.merge_core(trans, out, lhs_inp, rhs_inp)
    }

    fn split(&self, lhs_out: DartId, rhs_out: DartId, inp: DartId) {
        atomically(|trans| self.split_core(trans, lhs_out, rhs_out, inp));
    }

    fn split_transac(
        &self,
        trans: &mut Transaction,
        lhs_out: DartId,
        rhs_out: DartId,
        inp: DartId,
    ) -> Result<(), StmError> {
        self.split_core(trans, lhs_out, rhs_out, inp)
    }
}

impl<A: AttributeBind + AttributeUpdate> AttributeStorage<A> for AttrSparseVec<A> {
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
impl<T: AttributeBind + AttributeUpdate> AttrSparseVec<T> {
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
