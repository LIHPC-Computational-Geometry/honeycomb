//! Generic attributes implementation
//!
//! This module contains all code used to handle attribute genericity in the context of mesh
//! representation through combinatorial maps embedded data.

// ------ IMPORTS

use crate::OrbitPolicy;

// ------ CONTENT

/// Generic attribute trait for logical behavior
///
/// This trait can be implemented for a given attribute in order to define the behavior to
/// follow when (un)sewing operations result in an update of the attribute.
///
/// # Example
///
/// For an intensive property of a system (e.g. a temperature), a dummy implementation would look
/// like this:
///
/// ```rust
/// use honeycomb_core::AttributeLogic;
///
/// #[derive(Copy, Clone)]
/// pub struct Temperature {
///     pub val: f32
/// }
///
/// impl AttributeLogic for Temperature {
///     fn merge(lhs: Self, rhs: Self) -> Self {
///         Temperature { val: (lhs.val + rhs.val) / 2.0 }
///     }
///
///     fn split(lhs: Self) -> (Self, Self) {
///         (lhs, lhs)
///     }
///
///     fn merge_undefined(lhs: Option<Self>) -> Self {
///         lhs.unwrap_or(Temperature { val: 0.0 })
///     }
/// }
/// ```
pub trait AttributeLogic: Sized {
    /// Merging routine, i.e. how to obtain the new attribute value from the two existing ones.
    fn merge(lhs: Self, rhs: Self) -> Self;

    /// Splitting routine, i.e. how to obtain the two attributes from a single one.
    fn split(lhs: Self) -> (Self, Self);

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
    fn merge_undefined(lhs: Option<Self>) -> Self {
        lhs.unwrap()
    }
}

/// Generic attribute trait for support description
///
/// This trait can be implemented for a given attribute in order to hint at which components of
/// the map the attribute is bound.
///
/// # Example
///
/// Using the same context as the for the [AttributeLogic] example, we can associate temperature
/// to faces if we're modeling a 2D mesh:
///
/// ```rust
/// use honeycomb_core::{AttributeSupport, OrbitPolicy};
///
/// #[derive(Copy, Clone)]
/// pub struct Temperature {
///     pub val: f32
/// }
///
/// impl AttributeSupport for Temperature {
///     fn binds_to(&self) -> OrbitPolicy {
///         OrbitPolicy::Face
///     }
/// }
/// ```
pub trait AttributeSupport: Sized {
    /// Return an [OrbitPolicy] that can be used to identify the kind of topological entity to
    /// which the attribute is associated.
    fn binds_to(&self) -> OrbitPolicy;
}

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}