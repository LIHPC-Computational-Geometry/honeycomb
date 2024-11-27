//! Generic attributes implementation
//!
//! This module contains all code used to handle attribute genericity in the context of mesh
//! representation through combinatorial maps embedded data.

// ------ IMPORTS

use crate::{
    cmap::CMapResult,
    prelude::{DartIdType, OrbitPolicy},
};
use downcast_rs::{impl_downcast, Downcast};
use std::any::Any;
use std::fmt::Debug;
use stm::{atomically, StmResult, Transaction};

// ------ CONTENT

/// Generic attribute trait for logical behavior description
///
/// This trait can be implemented for a given attribute in order to define the behavior to
/// follow when (un)sewing operations result in an update of the attribute.
///
/// # Example
///
/// For an intensive property of a system (e.g. a temperature), an implementation would look
/// like this:
///
/// ```rust
/// use honeycomb_core::prelude::AttributeUpdate;
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// pub struct Temperature {
///     pub val: f32
/// }
///
/// impl AttributeUpdate for Temperature {
///     fn merge(attr1: Self, attr2: Self) -> Self {
///         Temperature { val: (attr1.val + attr2.val) / 2.0 }
///     }
///
///     fn split(attr: Self) -> (Self, Self) {
///         (attr, attr)
///     }
///
///     fn merge_incomplete(attr: Self) -> Self {
///         Temperature { val: attr.val / 2.0 }
///     }
///
///     fn merge_from_none() -> Option<Self> {
///         Some(Temperature { val: 0.0 })
///     }
/// }
///
/// let t1 = Temperature { val: 273.0 };
/// let t2 = Temperature { val: 298.0 };
///
/// let t_new = AttributeUpdate::merge(t1, t2); // use AttributeUpdate::_
/// let t_ref = Temperature { val: 285.5 };
///
/// assert_eq!(Temperature::split(t_new), (t_ref, t_ref)); // or Temperature::_
/// ```
pub trait AttributeUpdate: Sized + Send + Sync + Clone + Copy {
    /// Merging routine, i.e. how to obtain the new attribute value from the two existing ones.
    fn merge(attr1: Self, attr2: Self) -> Self;

    /// Splitting routine, i.e. how to obtain the two attributes from a single one.
    fn split(attr: Self) -> (Self, Self);

    /// Fallback merging routine, i.e. how to obtain the new attribute value from a single existing
    /// value.
    ///
    /// The default implementation simply returns the passed value.
    fn merge_incomplete(attr: Self) -> Self {
        attr
    }

    /// Fallback merging routine, i.e. how to obtain the new attribute value from no existing
    /// value.
    ///
    /// The default implementation return `None`.
    #[allow(clippy::must_use_candidate)]
    fn merge_from_none() -> Option<Self> {
        None
    }
}

/// Generic attribute trait for support description
///
/// This trait can be implemented for a given attribute in order to hint at which components of
/// the map the attribute is bound.
///
/// # Example
///
/// Using the same context as the for the [`AttributeUpdate`] example, we can associate temperature
/// to faces if we're modeling a 2D mesh:
///
/// ```rust
/// use honeycomb_core::prelude::{AttributeBind, AttributeUpdate, FaceIdType, OrbitPolicy};
/// use honeycomb_core::attributes::AttrSparseVec;
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// pub struct Temperature {
///     pub val: f32
/// }
/// # impl AttributeUpdate for Temperature {
/// #     fn merge(attr1: Self, attr2: Self) -> Self {
/// #         Temperature { val: (attr1.val + attr2.val) / 2.0 }
/// #     }
/// #
/// #     fn split(attr: Self) -> (Self, Self) {
/// #         (attr, attr)
/// #     }
/// #
/// #     fn merge_incomplete(attr: Self) -> Self {
/// #         Temperature { val: attr.val / 2.0 }
/// #     }
/// #
/// #     fn merge_from_none() -> Option<Self> {
/// #         Some(Temperature { val: 0.0 })
/// #     }
/// # }
///
/// impl AttributeBind for Temperature {
///     type StorageType = AttrSparseVec<Self>;
///     type IdentifierType = FaceIdType;
///     const BIND_POLICY: OrbitPolicy = OrbitPolicy::Face;
/// }
/// ```
pub trait AttributeBind: Debug + Sized + Any {
    /// Storage type used for the attribute.
    type StorageType: AttributeStorage<Self>;

    /// Identifier type of the entity the attribute is bound to.
    type IdentifierType: From<DartIdType> + num_traits::ToPrimitive + Clone;

    /// [`OrbitPolicy`] determining the kind of topological entity to which the attribute
    /// is associated.
    const BIND_POLICY: OrbitPolicy;
}

/// Common trait implemented by generic attribute storages.
///
/// This trait contain attribute-agnostic function & methods.
///
/// ### Note on force / regular / try semantics
///
/// We define three variants of split and merge methods (same as sews / unsews): `force`, regular,
/// and `try`. Their goal is to provide different degrees of control vs convenience when using
/// these operations. Documentation of each method shortly explains their individual quirks,
/// below is a table summarizing the differences:
///
/// | variant | description |
/// |---------| ----------- |
/// | `try`   | defensive impl, only succeding if the attribute operation is successful & the transaction isn't invalidated |
/// | regular | regular impl, which uses attribute fallback policies and will fail only if the transaction is invalidated   |
/// | `force` | convenience impl, which wraps the regular impl in a transaction that retries until success                  |
///
pub trait UnknownAttributeStorage: Any + Debug + Downcast {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- Initial length/capacity of the storage. It should correspond to
    ///   the upper bound of IDs used to index the attribute's values, i.e. the number of darts
    ///   including the null dart.
    ///
    /// # Return
    ///
    /// Return a [Self] instance which yields correct accesses over the ID range `0..length`.
    #[must_use = "constructed object is not used, consider removing this function call"]
    fn new(length: usize) -> Self
    where
        Self: Sized;

    /// Extend the storage's length
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- length of which the storage should be extended.
    fn extend(&mut self, length: usize);

    /// Return the number of stored attributes, i.e. the number of used slots in the storage, not
    /// its length.
    #[must_use = "returned value is not used, consider removing this method call"]
    fn n_attributes(&self) -> usize;

    // regular

    #[allow(clippy::missing_errors_doc)]
    /// Merge attributes at specified index
    ///
    /// # Arguments
    ///
    /// - `trans: &mut Transaction` -- Transaction used for synchronization.
    /// - `out: DartIdentifier` -- Identifier to associate the result with.
    /// - `lhs_inp: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `rhs_inp: DartIdentifier` -- Identifier of the other attribute value to merge.
    ///
    /// # Behavior (pseudo-code)
    ///
    /// ```text
    /// let new_val = match (attributes.remove(lhs_inp), attributes.remove(rhs_inp)) {
    ///     (Some(v1), Some(v2)) => AttributeUpdate::merge(v1, v2),
    ///     (Some(v), None) | (None, Some(v)) => AttributeUpdate::merge_undefined(Some(v)),
    ///     None, None => AttributeUpdate::merge_undefined(None),
    /// }
    /// attributes.set(out, new_val);
    /// ```
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transacction passed as argument. The result should not be processed manually.
    fn merge(
        &self,
        trans: &mut Transaction,
        out: DartIdType,
        lhs_inp: DartIdType,
        rhs_inp: DartIdType,
    ) -> StmResult<()>;

    #[allow(clippy::missing_errors_doc)]
    /// Split attribute to specified indices
    ///
    /// # Arguments
    ///
    /// - `trans: &mut Transaction` -- Transaction used for synchronization.
    /// - `lhs_out: DartIdentifier` -- Identifier to associate the result with.
    /// - `rhs_out: DartIdentifier` -- Identifier to associate the result with.
    /// - `inp: DartIdentifier` -- Identifier of the attribute value to split.
    ///
    /// # Behavior pseudo-code
    ///
    /// ```text
    /// (val_lhs, val_rhs) = AttributeUpdate::split(attributes.remove(inp).unwrap());
    /// attributes[lhs_out] = val_lhs;
    /// attributes[rhs_out] = val_rhs;
    /// ```
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually.
    fn split(
        &self,
        trans: &mut Transaction,
        lhs_out: DartIdType,
        rhs_out: DartIdType,
        inp: DartIdType,
    ) -> StmResult<()>;

    // force

    /// Merge attributes at specified index
    ///
    /// This variant is equivalent to `merge`, but internally uses a transaction that will be
    /// retried until validated.
    fn force_merge(&self, out: DartIdType, lhs_inp: DartIdType, rhs_inp: DartIdType) {
        atomically(|trans| self.merge(trans, out, lhs_inp, rhs_inp));
    }

    /// Split attribute to specified indices
    ///
    /// This variant is equivalent to `split`, but internally uses a transaction that will be
    /// retried until validated.
    fn force_split(&self, lhs_out: DartIdType, rhs_out: DartIdType, inp: DartIdType) {
        atomically(|trans| self.split(trans, lhs_out, rhs_out, inp));
    }

    // try

    /// Merge attributes at specified index
    ///
    /// # Errors
    ///
    /// This method will fail, returning an error, if:
    /// - the transaction cannot be completed
    /// - the merge fails (e.g. because one merging value is missing)
    ///
    /// The returned error can be used in conjunction with transaction control to avoid any
    /// modifications in case of failure at attribute level. The user can then choose, through its
    /// transaction control policy, to retry or abort as he wishes.
    fn try_merge(
        &self,
        trans: &mut Transaction,
        out: DartIdType,
        lhs_inp: DartIdType,
        rhs_inp: DartIdType,
    ) -> CMapResult<()>;

    /// Split attribute to specified indices
    ///
    /// # Errors
    ///
    /// This method will fail, returning an error, if:
    /// - the transaction cannot be completed
    /// - the split fails (e.g. because there is no value to split from)
    ///
    /// The returned error can be used in conjunction with transaction control to avoid any
    /// modifications in case of failure at attribute level. The user can then choose, through its
    /// transaction control policy, to retry or abort as he wishes.
    fn try_split(
        &self,
        trans: &mut Transaction,
        lhs_out: DartIdType,
        rhs_out: DartIdType,
        inp: DartIdType,
    ) -> CMapResult<()>;
}

impl_downcast!(UnknownAttributeStorage);

/// Common trait implemented by generic attribute storages.
///
/// This trait contain attribute-specific methods.
///
/// The documentation of this trait describe the behavior each function & method should have. "ID"
/// and "index" are used interchangeably.
///
/// ### Note on force / regular semantics
///
/// We define two variants of read / write / remove methods: `force` and regular. Their goal is to
/// provide different degrees of control vs convenience when using these operations. Documentation
/// of each method shortly explains their individual quirks, below is a table summarizing the
/// differences:
///
/// | variant | description                                                                                |
/// |---------| ------------------------------------------------------------------------------------------ |
/// | regular | regular impl, which will fail if the transaction fails                                     |
/// | `force` | convenience impl, which wraps the regular impl in a transaction that retries until success |
pub trait AttributeStorage<A: AttributeBind>: UnknownAttributeStorage {
    #[allow(clippy::missing_errors_doc)]
    /// Read the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `trans: &mut Transaction` -- Transaction used for synchronization.
    /// - `index: A::IdentifierType` -- Cell index.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually,
    /// only used via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn read(&self, trans: &mut Transaction, id: A::IdentifierType) -> StmResult<Option<A>>;

    #[allow(clippy::missing_errors_doc)]
    /// Write the value of an element at a given index and return the old value.
    ///
    /// # Arguments
    ///
    /// - `trans: &mut Transaction` -- Transaction used for synchronization.
    /// - `index: A::IdentifierType` -- Cell index.
    /// - `val: A` -- Attribute value.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually,
    /// only used via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn write(&self, trans: &mut Transaction, id: A::IdentifierType, val: A)
        -> StmResult<Option<A>>;

    #[allow(clippy::missing_errors_doc)]
    /// Remove the value at a given index and return it.
    ///
    /// # Arguments
    ///
    /// - `trans: &mut Transaction` -- Transaction used for synchronization.
    /// - `index: A::IdentifierType` -- Cell index.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually,
    /// only used via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn remove(&self, trans: &mut Transaction, id: A::IdentifierType) -> StmResult<Option<A>>;

    /// Read the value of an element at a given index.
    ///
    /// This variant is equivalent to `read`, but internally uses a transaction that will be
    /// retried until validated.
    fn force_read(&self, id: A::IdentifierType) -> Option<A> {
        atomically(|trans| self.read(trans, id.clone()))
    }

    /// Write the value of an element at a given index and return the old value.
    ///
    /// This variant is equivalent to `write`, but internally uses a transaction that will be
    /// retried until validated.
    fn force_write(&self, id: A::IdentifierType, val: A) -> Option<A>;

    /// Remove the value at a given index and return it.
    ///
    /// This variant is equivalent to `remove`, but internally uses a transaction that will be
    /// retried until validated.
    fn force_remove(&self, id: A::IdentifierType) -> Option<A> {
        atomically(|trans| self.remove(trans, id.clone()))
    }
}
