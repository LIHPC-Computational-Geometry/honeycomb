//! Crate-level common definitions

// ------ CONTENT

// --- darts

use std::sync::atomic::{AtomicU32, Ordering};

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type PDartIdentifier = AtomicU32;

pub fn is_null(d: &PDartIdentifier) -> bool {
    d.load(Ordering::Relaxed) == 0 // isn't there better?
}
