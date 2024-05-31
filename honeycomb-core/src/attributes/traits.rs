//! Generic attributes implementation
//!
//! This module contains all code used to handle attribute genericity in the context of mesh
//! representation through combinatorial maps embedded data.

// ------ IMPORTS

use crate::OrbitPolicy;
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
    type IdentifierType: num::ToPrimitive;

    /// Return an [`OrbitPolicy`] that can be used to identify the kind of topological entity to
    /// which the attribute is associated.
    fn binds_to<'a>() -> OrbitPolicy<'a>;
}

pub trait AttributeStorage<A: AttributeBind>: Debug + Any {
    fn new(length: usize) -> Self
    where
        Self: Sized;

    fn extend(&mut self, length: usize);

    fn n_attributes(&self) -> usize;

    fn set(&mut self, id: A::IdentifierType, val: A);

    fn insert(&mut self, id: A::IdentifierType, val: A);

    fn get(&self, id: A::IdentifierType) -> Option<A>;

    fn replace(&mut self, id: A::IdentifierType, val: A) -> Option<A>;

    fn remove(&mut self, id: A::IdentifierType) -> Option<A>;
}
