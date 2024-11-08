//! Generic attributes implementation
//!
//! This module contains all code used to handle attribute genericity in the context of mesh
//! representation through combinatorial maps embedded data.

// ------ IMPORTS

use crate::cmap::{CellId, DartId, OrbitPolicy};
use downcast_rs::{impl_downcast, Downcast};
use std::any::Any;
use std::fmt::Debug;
use stm::{StmError, Transaction};

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
pub trait AttributeUpdate: Sized + Send + Sync {
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
/// use honeycomb_core::prelude::{AttributeBind, AttributeUpdate, FaceIdentifier, OrbitPolicy};
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
///     type IdentifierType = FaceIdentifier;
///     const BIND_POLICY: OrbitPolicy = OrbitPolicy::Face;
/// }
/// ```
pub trait AttributeBind: Debug + Sized + Any + Clone + Copy {
    /// Storage type used for the attribute.
    type StorageType: AttributeStorage<Self>;

    /// Identifier type of the entity the attribute is bound to.
    type IdentifierType: CellId + Clone;

    /// [`OrbitPolicy`] determining the kind of topological entity to which the attribute
    /// is associated.
    const BIND_POLICY: OrbitPolicy;
}

/// Common trait implemented by generic attribute storages.
///
/// This trait contain attribute-agnostic function & methods.
///
/// The documentation of this trait describe the behavior each function & method should have.
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

    /// Merge attributes at specified index
    ///
    /// This method should serve as a wire to either `AttributeUpdate::merge`
    /// or `AttributeUpdate::merge_undefined` after removing the values we wish to merge from
    /// the storage.
    ///
    /// # Arguments
    ///
    /// - `out: DartId` -- Identifier to associate the result with.
    /// - `lhs_inp: DartId` -- Identifier of one attribute value to merge.
    /// - `rhs_inp: DartId` -- Identifier of the other attribute value to merge.
    ///
    /// # Behavior pseudo-code
    ///
    /// ```text
    /// let new_val = match (attributes.remove(lhs_inp), attributes.remove(rhs_inp)) {
    ///     (Some(v1), Some(v2)) => AttributeUpdate::merge(v1, v2),
    ///     (Some(v), None) | (None, Some(v)) => AttributeUpdate::merge_undefined(Some(v)),
    ///     None, None => AttributeUpdate::merge_undefined(None),
    /// }
    /// attributes.set(out, new_val);
    /// ```
    fn merge(&self, out: DartId, lhs_inp: DartId, rhs_inp: DartId);

    #[allow(clippy::missing_errors_doc)]
    /// Transactional `merge`
    ///
    /// # Result / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transacction passed as argument. The result should not be processed manually.
    fn merge_transac(
        &self,
        trans: &mut Transaction,
        out: DartId,
        lhs_inp: DartId,
        rhs_inp: DartId,
    ) -> Result<(), StmError>;

    /// Split attribute to specified indices
    ///
    /// This method should serve as a wire to `AttributeUpdate::split` after removing the value
    /// we want to split from the storage.
    ///
    /// # Arguments
    ///
    /// - `lhs_out: DartId` -- Identifier to associate the result with.
    /// - `rhs_out: DartId` -- Identifier to associate the result with.
    /// - `inp: DartId` -- Identifier of the attribute value to split.
    ///
    /// # Behavior pseudo-code
    ///
    /// ```text
    /// (val_lhs, val_rhs) = AttributeUpdate::split(attributes.remove(inp).unwrap());
    /// attributes[lhs_out] = val_lhs;
    /// attributes[rhs_out] = val_rhs;
    /// ```
    fn split(&self, lhs_out: DartId, rhs_out: DartId, inp: DartId);

    #[allow(clippy::missing_errors_doc)]
    /// Transactional `split`
    ///
    /// # Result / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transacction passed as argument. The result should not be processed manually.
    fn split_transac(
        &self,
        trans: &mut Transaction,
        lhs_out: DartId,
        rhs_out: DartId,
        inp: DartId,
    ) -> Result<(), StmError>;
}

impl_downcast!(UnknownAttributeStorage);

/// Common trait implemented by generic attribute storages.
///
/// This trait contain attribute-specific methods.
///
/// The documentation of this trait describe the behavior each function & method should have. "ID"
/// and "index" are used interchangeably.
pub trait AttributeStorage<A: AttributeBind>: UnknownAttributeStorage {
    /// Setter
    ///
    /// Set the value of an element at a given index. This operation is not affected by the initial
    /// state of the edited entry.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    /// - `val: A` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn set(&self, id: A::IdentifierType, val: A);

    #[allow(clippy::missing_errors_doc)]
    /// Transactional `set`
    ///
    /// # Result / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transacction passed as argument. The result should not be processed manually.
    fn set_transac(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
        val: A,
    ) -> Result<(), StmError>;

    /// Setter
    ///
    /// Insert a value at a given empty index.
    /// Otherwise, see [#Panics] section for more information.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    /// - `val: A` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method:
    /// - **should panic if there is already a value associated to the specified index**
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn insert(&self, id: A::IdentifierType, val: A) {
        assert!(self.get(id.clone()).is_none());
        self.set(id, val);
    }

    #[allow(clippy::missing_errors_doc)]
    /// Transactional `insert`
    ///
    /// # Result / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transacction passed as argument. The result should not be processed manually.
    fn insert_transac(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
        val: A,
    ) -> Result<(), StmError> {
        assert!(self.get(id.clone()).is_none());
        self.set_transac(trans, id, val)
    }

    /// Getter
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// The method should return:
    /// - `Some(val: A)` if there is an attribute associated with the specified index,
    /// - `None` if there is not.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn get(&self, id: A::IdentifierType) -> Option<A>;

    #[allow(clippy::missing_errors_doc)]
    /// Transactional `get`
    ///
    /// # Result / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transacction passed as argument. The result should not be processed manually.
    fn get_transac(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
    ) -> Result<Option<A>, StmError>;

    /// Setter
    ///
    /// Replace the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    /// - `val: A` -- Attribute value.
    ///
    /// # Return
    ///
    /// The method should return:
    /// - `Some(val_old: A)` if there was an attribute associated with the specified index,
    /// - `None` if there is not.
    ///
    /// In both cases, the new value should be set to the one specified as argument.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn replace(&self, id: A::IdentifierType, val: A) -> Option<A>;

    #[allow(clippy::missing_errors_doc)]
    /// Transactional `replace`
    ///
    /// # Result / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transacction passed as argument. The result should not be processed manually.
    fn replace_transac(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
        val: A,
    ) -> Result<Option<A>, StmError>;

    /// Remove an item from the storage and return it
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// The method should return:
    /// - `Some(val: A)` if there was an attribute associated with the specified index,
    /// - `None` if there is not.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    fn remove(&self, id: A::IdentifierType) -> Option<A>;

    #[allow(clippy::missing_errors_doc)]
    /// Transactional `remove`
    ///
    /// # Result / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transacction passed as argument. The result should not be processed manually.
    fn remove_transac(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
    ) -> Result<Option<A>, StmError>;
}
