//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

// ------ IMPORTS

// ------ CONTENT

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type DartIdentifier = u32;

pub const NULL_DART_ID: DartIdentifier = 0;

/// Type definition for vertex identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type VertexIdentifier = u32;

/// Type definition for vertex identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type EdgeIdentifier = u32;

/// Type definition for face identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type FaceIdentifier = u32;

/// Type definition for volume identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type VolumeIdentifier = u32;

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
