//! Main error type

use crate::{attributes::AttributeError, cmap::DartIdType};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LinkError {
    #[error("cannot link {1} to {2}: b{0}({1}) != NULL")]
    NonFreeBase(u8, DartIdType, DartIdType),
    #[error("cannot link {1} to {2}: b{0}({2}) != NULL")]
    NonFreeImage(u8, DartIdType, DartIdType),
    #[error("cannot unlink {1}: b{0}({1}) == NULL")]
    AlreadyFree(u8, DartIdType),
}

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
