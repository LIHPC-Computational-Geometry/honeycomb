//! Generic attributes implementation
//!
//!

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
/// };
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
    fn merge(lhs: Self, rhs: Self) -> Self;

    fn split(lhs: Self) -> (Self, Self);

    fn merge_undefined(lhs: Option<Self>) -> Self {
        lhs.unwrap() // todo: choose a policy for default behavior
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
/// };
///
/// impl AttributeSupport for Temperature {
///     fn binds_to(&self) -> OrbitPolicy {
///         OrbitPolicy::Face
///     }
/// }
/// ```
pub trait AttributeSupport: Sized {
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
