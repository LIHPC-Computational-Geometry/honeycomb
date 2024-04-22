//! Attribute modeling code
//!
//! This module contains all code related to generic attribute modelling and handling.

// ------ MODULE DECLARATIONS

pub mod collections;
pub mod traits;

// ------ TESTS

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Temperature {
    pub val: f32,
}

#[cfg(test)]
impl crate::AttributeUpdate for Temperature {
    fn merge(attr1: Self, attr2: Self) -> Self {
        Temperature {
            val: (attr1.val + attr2.val) / 2.0,
        }
    }

    fn split(attr: Self) -> (Self, Self) {
        (attr, attr)
    }

    fn merge_undefined(attr: Option<Self>) -> Self {
        attr.unwrap_or(Temperature { val: 0.0 })
    }
}

#[cfg(test)]
impl crate::AttributeBind for Temperature {
    type IdentifierType = crate::FaceIdentifier;
    fn binds_to<'a>() -> crate::OrbitPolicy<'a> {
        crate::OrbitPolicy::Face
    }
}

#[cfg(test)]
impl From<f32> for Temperature {
    fn from(val: f32) -> Self {
        Self { val }
    }
}
