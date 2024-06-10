//! Generic attributes implementation
//!
//! This module contains all code used to handle attribute genericity in the context of mesh
//! representation through combinatorial maps embedded data.

// ------ IMPORTS

use crate::{DartIdentifier, OrbitPolicy};
use std::any::Any;
use std::fmt::Debug;

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
/// use honeycomb_core::AttributeUpdate;
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
///     fn merge_undefined(attr: Option<Self>) -> Self {
///         attr.unwrap_or(Temperature { val: 0.0 })
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
pub trait AttributeUpdate: Sized {
    /// Merging routine, i.e. how to obtain the new attribute value from the two existing ones.
    fn merge(attr1: Self, attr2: Self) -> Self;

    /// Splitting routine, i.e. how to obtain the two attributes from a single one.
    fn split(attr: Self) -> (Self, Self);

    /// Fallback merging routine, i.e. how to obtain the new attribute value from potentially
    /// undefined instances.
    ///
    /// The default implementation may panic if no attribute can be used to create a value. The
    /// reason for that is as follows:
    ///
    /// This trait and its methods were designed with the (un)sewing operation in mind. Their
    /// purpose is to simplify the code needed to propagate updates of attributes affected by the
    /// (un)sewing operation. Considering this context, as well as the definition of (un)linking
    /// operations, this panic seems reasonable: If the darts you are sewing have totally undefined
    /// attributes, you should most likely be linking them instead of sewing.
    fn merge_undefined(attr: Option<Self>) -> Self {
        attr.unwrap()
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
/// use honeycomb_core::{AttributeBind, AttributeUpdate, FaceIdentifier, OrbitPolicy, AttrSparseVec};
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
/// #     fn merge_undefined(attr: Option<Self>) -> Self {
/// #         attr.unwrap_or(Temperature { val: 0.0 })
/// #     }
/// # }
///
/// impl AttributeBind for Temperature {
///     # type StorageType = AttrSparseVec<Self>;
///     type IdentifierType = FaceIdentifier;
///
///     fn binds_to<'a>() -> OrbitPolicy<'a> {
///         OrbitPolicy::Face
///     }
/// }
/// ```
pub trait AttributeBind: Debug + Sized + Any {
    /// Storage type used for the attribute.
    type StorageType: AttributeStorage<Self>;

    /// Identifier type of the entity the attribute is bound to.
    type IdentifierType: From<DartIdentifier> + num::ToPrimitive + Clone;

    /// Return an [`OrbitPolicy`] that can be used to identify the kind of topological entity to
    /// which the attribute is associated.
    fn binds_to<'a>() -> OrbitPolicy<'a>;
}

/// Common trait implemented by generic attribute storages.
///
/// This trait contain attribute-agnostic function & methods.
///
/// The documentation of this trait describe the behavior each function & method should have.
pub trait UnknownAttributeStorage: Debug + Any {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- Initial length/capacity of the storage. It should correspond to
    /// the upper bound of IDs used to index the attribute's values, i.e. the number of darts
    /// including the null dart.
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
    /// - `out: DartIdentifier` -- Identifier to associate the result with.
    /// - `lhs_inp: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `rhs_inp: DartIdentifier` -- Identifier of the other attribute value to merge.
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
    fn merge(&mut self, out: DartIdentifier, lhs_inp: DartIdentifier, rhs_inp: DartIdentifier);

    /// Split attribute to specified indices
    ///
    /// This method should serve as a wire to `AttributeUpdate::split` after removing the value
    /// we want to split from the storage.
    ///
    /// # Arguments
    ///
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
    fn split(&mut self, lhs_out: DartIdentifier, rhs_out: DartIdentifier, inp: DartIdentifier);
}

/// Common trait implemented by generic attribute storages.
///
/// This trait contain attribute-specific methods.
///
/// The documentation of this trait describe the behavior each function & method should have.
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
    fn set(&mut self, id: A::IdentifierType, val: A);

    /// Setter
    ///
    /// Insert a value at a given empty index.
    /// Otherwise, see [Panics] section for more information.
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
    fn insert(&mut self, id: A::IdentifierType, val: A) {
        assert!(self.get(id.clone()).is_none());
        self.set(id, val);
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
    fn replace(&mut self, id: A::IdentifierType, val: A) -> Option<A>;

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
    fn remove(&mut self, id: A::IdentifierType) -> Option<A>;
}
