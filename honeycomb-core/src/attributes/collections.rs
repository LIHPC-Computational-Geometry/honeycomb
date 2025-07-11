//! Attribute storage structures
//!
//! This module contains all code used to describe custom collections used to store attributes
//! (see [`AttributeBind`], [`AttributeUpdate`]).

use num_traits::ToPrimitive;
#[cfg(feature = "par-internals")]
use rayon::prelude::*;

use crate::attributes::{
    AttributeBind, AttributeError, AttributeStorage, AttributeUpdate, UnknownAttributeStorage,
};
use crate::cmap::DartIdType;
use crate::stm::{StmClosureResult, TVar, Transaction, TransactionClosureResult, abort};

/// Custom storage structure
///
/// **This structure is not meant to be used directly** but with the [`AttributeBind`] trait.
///
/// The structure is used to store user-defined attributes using a vector of `Option<T>` items.
/// This implementation should favor access logic over locality of reference.
///
/// # Generics
///
/// - `T: AttributeBind + AttributeUpdate` -- Type of the stored attributes.
///
#[derive(Debug)]
pub struct AttrSparseVec<T: AttributeBind + AttributeUpdate> {
    /// Inner storage.
    data: Vec<TVar<Option<T>>>,
}

unsafe impl<A: AttributeBind + AttributeUpdate> Send for AttrSparseVec<A> {}
unsafe impl<A: AttributeBind + AttributeUpdate> Sync for AttrSparseVec<A> {}

impl<A: AttributeBind + AttributeUpdate> UnknownAttributeStorage for AttrSparseVec<A> {
    #[cfg(not(feature = "par-internals"))]
    fn new(length: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            data: (0..length).map(|_| TVar::new(None)).collect(),
        }
    }

    #[cfg(feature = "par-internals")]
    fn new(length: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            data: (0..length)
                .into_par_iter()
                .map(|_| TVar::new(None))
                .collect(),
        }
    }

    #[cfg(not(feature = "par-internals"))]
    fn extend(&mut self, length: usize) {
        self.data.extend((0..length).map(|_| TVar::new(None)));
    }

    #[cfg(feature = "par-internals")]
    fn extend(&mut self, length: usize) {
        self.data
            .par_extend((0..length).into_par_iter().map(|_| TVar::new(None)));
    }

    fn n_attributes(&self) -> usize {
        self.data
            .iter()
            .filter(|v| v.read_atomic().is_some())
            .count()
    }

    fn merge(
        &self,
        trans: &mut Transaction,
        out: DartIdType,
        lhs_inp: DartIdType,
        rhs_inp: DartIdType,
    ) -> TransactionClosureResult<(), AttributeError> {
        let new_v = match (
            self.data[lhs_inp as usize].read(trans)?,
            self.data[rhs_inp as usize].read(trans)?,
        ) {
            (Some(v1), Some(v2)) => AttributeUpdate::merge(v1, v2),
            (Some(v), None) | (None, Some(v)) => AttributeUpdate::merge_incomplete(v),
            (None, None) => AttributeUpdate::merge_from_none(),
        };
        match new_v {
            Ok(v) => {
                self.data[rhs_inp as usize].write(trans, None)?;
                self.data[lhs_inp as usize].write(trans, None)?;
                self.data[out as usize].write(trans, Some(v))?;
                Ok(())
            }
            Err(e) => abort(e),
        }
    }

    fn split(
        &self,
        trans: &mut Transaction,
        lhs_out: DartIdType,
        rhs_out: DartIdType,
        inp: DartIdType,
    ) -> TransactionClosureResult<(), AttributeError> {
        let res = if let Some(val) = self.data[inp as usize].read(trans)? {
            AttributeUpdate::split(val)
        } else {
            AttributeUpdate::split_from_none()
        };
        match res {
            Ok((lhs_val, rhs_val)) => {
                self.data[inp as usize].write(trans, None)?;
                self.data[lhs_out as usize].write(trans, Some(lhs_val))?;
                self.data[rhs_out as usize].write(trans, Some(rhs_val))?;
                Ok(())
            }
            Err(e) => abort(e),
        }
    }
}

impl<A: AttributeBind + AttributeUpdate> AttributeStorage<A> for AttrSparseVec<A> {
    fn write(
        &self,
        trans: &mut Transaction,
        id: <A as AttributeBind>::IdentifierType,
        val: A,
    ) -> StmClosureResult<Option<A>> {
        self.data[id.to_usize().unwrap()].replace(trans, Some(val))
    }

    fn read(
        &self,
        trans: &mut Transaction,
        id: <A as AttributeBind>::IdentifierType,
    ) -> StmClosureResult<Option<A>> {
        self.data[id.to_usize().unwrap()].read(trans)
    }

    fn remove(
        &self,
        trans: &mut Transaction,
        id: <A as AttributeBind>::IdentifierType,
    ) -> StmClosureResult<Option<A>> {
        self.data[id.to_usize().unwrap()].replace(trans, None)
    }
}
