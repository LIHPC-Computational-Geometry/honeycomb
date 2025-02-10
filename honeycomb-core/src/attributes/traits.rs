//! Generic attributes implementation
//!
//! This module contains all code used to handle attribute genericity in the context of mesh
//! representation through combinatorial maps embedded data.

// ------ IMPORTS

use crate::attributes::AttributeError;
use crate::prelude::{DartIdType, OrbitPolicy};
use crate::stm::{atomically, StmClosureResult, Transaction};

use downcast_rs::{impl_downcast, Downcast};
use fast_stm::TransactionClosureResult;

use std::any::{type_name, Any};
use std::fmt::Debug;

// ------ CONTENT

/// # Generic attribute trait
///
/// This trait is used to describe how a values of a given attribute are merged and split during
/// sewing and unsewing operations.
///
/// ## Example
///
/// A detailed example is provided in the [user guide][UG].
///
/// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/user-guide/usage/attributes.html
pub trait AttributeUpdate: Sized + Send + Sync + Clone + Copy {
    /// Merging routine, i.e. how to obtain a new value from two existing ones.
    ///
    /// # Errors
    ///
    /// You may use [`AttributeError::FailedMerge`] to model a possible failure in your attribute
    /// mergin process.
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError>;

    /// Splitting routine, i.e. how to obtain the two new values from a single one.
    ///
    /// # Errors
    ///
    /// You may use [`AttributeError::FailedSplit`] to model a possible failure in your attribute
    /// mergin process.
    fn split(attr: Self) -> Result<(Self, Self), AttributeError>;

    #[allow(clippy::missing_errors_doc)]
    /// Fallback merging routine, i.e. how to obtain a new value from a single existing one.
    ///
    /// The returned value directly affects the behavior of sewing methods: For example, if this
    /// method returns an error for a given attribute, the `sew` method will fail. This allows the
    /// user to define some attribute-specific behavior and enable fallbacks when it makes sense.
    ///
    /// # Return / Errors
    ///
    /// The default implementation fails and returns [`AttributeError::InsufficientData`]. You may
    /// override the implementation and use [`AttributeError::FailedMerge`] to model another
    /// possible failure.
    fn merge_incomplete(_: Self) -> Result<Self, AttributeError> {
        Err(AttributeError::InsufficientData(
            "merge",
            type_name::<Self>(),
        ))
    }

    /// Fallback merging routine, i.e. how to obtain a new value from no existing one.
    ///
    /// The returned value directly affects the behavior of sewing methods: For example, if this
    /// method returns an error for a given attribute, the `sew` method will fail. This allows the
    /// user to define some attribute-specific behavior and enable fallbacks when it makes sense.
    ///
    /// # Errors
    ///
    /// The default implementation fails and returns [`AttributeError::InsufficientData`].
    fn merge_from_none() -> Result<Self, AttributeError> {
        Err(AttributeError::InsufficientData(
            "merge",
            type_name::<Self>(),
        ))
    }

    /// Fallback splitting routine, i.e. how to obtain two new values from no existing one.
    ///
    /// The returned value directly affects the behavior of sewing methods: For example, if this
    /// method returns an error for a given attribute, the `unsew` method will fail. This allows the
    /// user to define some attribute-specific behavior and enable fallbacks when it makes sense.
    /// value).
    ///
    /// # Errors
    ///
    /// The default implementation fails and returns [`AttributeError::InsufficientData`].
    fn split_from_none() -> Result<(Self, Self), AttributeError> {
        Err(AttributeError::InsufficientData(
            "split",
            type_name::<Self>(),
        ))
    }
}

/// # Generic attribute trait
///
/// This trait is used to describe how a given attribute binds to the map, and how it should be
/// stored in memory.
///
/// ## Example
///
/// A detailed example is provided in the [user guide][UG].
///
/// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/user-guide/usage/attributes.html
pub trait AttributeBind: Debug + Sized + Any {
    /// Storage type used for the attribute.
    type StorageType: AttributeStorage<Self>;

    /// Identifier type of the entity the attribute is bound to.
    type IdentifierType: From<DartIdType> + num_traits::ToPrimitive + Clone;

    /// [`OrbitPolicy`] determining the kind of topological entity to which the attribute
    /// is associated.
    const BIND_POLICY: OrbitPolicy;
}

/// # Generic attribute storage trait
///
/// This trait defines attribute-agnostic functions & methods. The documentation describes the
/// expected behavior of each item. “ID” and “index” are used interchangeably.
///
/// ### Note on force / regular / try semantics
///
/// <div class="warning">
/// This will be simplified in the near future, most likely with the deletion of force variants.
/// </div>
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
    #[must_use = "unused return value"]
    fn new(length: usize) -> Self
    where
        Self: Sized;

    /// Extend the storage's length
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- length of which the storage should be extended.
    fn extend(&mut self, length: usize);

    /// Return the number of stored attributes, i.e. the number of used slots in the storage (not
    /// its length).
    #[must_use = "unused return value"]
    fn n_attributes(&self) -> usize;

    // regular

    #[allow(clippy::missing_errors_doc)]
    /// Merge attributes to specified index
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
    /// validate the transaction passed as argument. Errors should not be processed manually.
    fn merge(
        &self,
        trans: &mut Transaction,
        out: DartIdType,
        lhs_inp: DartIdType,
        rhs_inp: DartIdType,
    ) -> StmClosureResult<()>;

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
    /// validate the transaction passed as argument. Errors should not be processed manually.
    fn split(
        &self,
        trans: &mut Transaction,
        lhs_out: DartIdType,
        rhs_out: DartIdType,
        inp: DartIdType,
    ) -> StmClosureResult<()>;

    // force

    /// Merge attributes to specified index
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

    /// Merge attributes to specified index
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
    ) -> TransactionClosureResult<(), AttributeError>;

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
    ) -> TransactionClosureResult<(), AttributeError>;
}

impl_downcast!(UnknownAttributeStorage);

/// # Generic attribute storage trait
///
/// This trait defines attribute-specific methods. The documentation describes the expected behavior
/// of each method. "ID" and "index" are used interchangeably.
///
/// Aside from the regular (transactional) read / write / remove methods, we provide `force`
/// variants which wraps regular methods in a transaction that retries until success. The main
/// purpose of these variants is to allow omitting transactions when they're not needed.
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
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn read(&self, trans: &mut Transaction, id: A::IdentifierType) -> StmClosureResult<Option<A>>;

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
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn write(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
        val: A,
    ) -> StmClosureResult<Option<A>>;

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
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn remove(&self, trans: &mut Transaction, id: A::IdentifierType)
        -> StmClosureResult<Option<A>>;

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
