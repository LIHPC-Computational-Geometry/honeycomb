//! Main error type

use crate::{attributes::AttributeError, cmap::DartIdType};

/// Dart allocation error struct
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("cannot reserve {0} darts: not enough unused darts")]
pub struct DartReservationError(pub usize);

/// Dart freeing error struct
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("cannot set dart {0} as unused: dart isn't free")]
pub struct DartReleaseError(pub DartIdType);

/// Link operation error enum
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LinkError {
    /// The base dart is not free.
    #[error("cannot link {1} to {2}: b{0}({1}) != NULL")]
    NonFreeBase(u8, DartIdType, DartIdType),
    /// The image dart is not free
    #[error("cannot link {1} to {2}: b{0}({2}) != NULL")]
    NonFreeImage(u8, DartIdType, DartIdType),
    /// The dart is already free.
    #[error("cannot unlink {1}: b{0}({1}) == NULL")]
    AlreadyFree(u8, DartIdType),
    /// The two orbits being linked have different structures.
    #[error("cannot 3-link {0} and {1}: faces do not have the same structure")]
    AsymmetricalFaces(DartIdType, DartIdType),
}

/// Sew operation error enum
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SewError {
    /// Geometry predicate failed verification.
    #[error("cannot {0}-sew darts {1} and {2} due to geometry predicates")]
    BadGeometry(u8, DartIdType, DartIdType),
    /// Dart link failed.
    #[error("inner link failed: {0}")]
    FailedLink(#[from] LinkError),
    /// Attribute operation failed.
    #[error("attribute operation failed: {0}")]
    FailedAttributeOp(#[from] AttributeError),
}
