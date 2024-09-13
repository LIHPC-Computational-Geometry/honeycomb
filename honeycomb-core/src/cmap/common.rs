//! Crate-level common definitions

// ------ CONTENT

// --- darts

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type DartIdentifier = u32;

/// Null value for dart identifiers
pub const NULL_DART_ID: DartIdentifier = 0;
