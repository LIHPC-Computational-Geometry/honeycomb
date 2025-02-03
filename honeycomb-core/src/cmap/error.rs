//! Main error type

use crate::{cmap::DartIdType, stm::StmError};

/// Convenience type alias
pub type CMapResult<T> = Result<T, CMapError>;

/// # Map-level error enum.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CMapError {
    /// STM transaction failed.
    #[error("transaction failed")]
    FailedTransaction(#[from] StmError),
    /// Attribute merge failed due to missing value(s).
    #[error("attribute merge failed: {0}")]
    FailedAttributeMerge(&'static str),
    /// Attribute split failed due to missing value.
    #[error("attribute split failed: {0}")]
    FailedAttributeSplit(&'static str),
    /// Geometry predicate failed verification.
    #[error("operation incompatible with map geometry: {0}")]
    IncorrectGeometry(&'static str),
    /// Accessed attribute isn't in the map storage.
    #[error("unknown attribute: {0}")]
    UnknownAttribute(&'static str),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LinkError {
    // FailedTransaction(/*#[from]*/ StmError),
    #[error("cannot link {0} to {1}: b1({0}) != NULL")]
    NonFreeBase(DartIdType, DartIdType),
    #[error("cannot link {0} to {1}: b0({1}) != NULL")]
    NonFreeImage(DartIdType, DartIdType),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SewError {
    // FailedTransaction(/*#[from]*/ StmError),
    /// Geometry predicate failed verification.
    #[error("operation incompatible with map geometry: {0}")]
    BadGeometry(&'static str),
    /// Dart link failed.
    #[error("inner link failed: {0}")]
    FailedLink(LinkError),
    /// Attribute merge failed due to missing value(s).
    #[error("attribute merge failed: {0}")]
    FailedMerge(&'static str),
    /// Attribute split failed due to missing value.
    #[error("attribute split failed: {0}")]
    FailedSplit(&'static str),
}
