//! Crate-level common definitions

// ------ CONTENT

// --- darts

use crate::cmap::NULL_DART_ID;
use std::sync::atomic::{AtomicU32, Ordering};

/// Type definition for dart identifiers
///
/// This is used for better control over memory usage and ID encoding.
pub type PDartIdentifier = AtomicU32;

/// Check if a given [`PDartIdentifier`] is null.
pub fn is_null(d: &PDartIdentifier) -> bool {
    d.load(Ordering::Relaxed) == NULL_DART_ID // isn't there better?
}
