//! Main error type

use crate::stm::StmError;

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
