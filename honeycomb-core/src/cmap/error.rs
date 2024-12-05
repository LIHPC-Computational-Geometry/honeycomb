//! Main error type

use stm::StmError;

/// Convenience type alias.
pub type CMapResult<T> = Result<T, CMapError>;

/// `CMap` error enum.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CMapError {
    /// STM transaction failed.
    #[error("transaction failed")]
    FailedTransaction(/*#[from]*/ StmError),
    /// Attribute merge failed due to missing value(s).
    #[error("attribute merge failed: {0}")]
    FailedAttributeMerge(&'static str),
    /// Attribute split failed due to missing value.
    #[error("attribute split failed: {0}")]
    FailedAttributeSplit(&'static str),
    /// Geometry check failed.
    #[error("operation incompatible with map geometry: {0}")]
    IncorrectGeometry(&'static str),
    /// No attribute value associated to specified ID.
    #[error("missing attribute: {0}")]
    MissingAttribute(&'static str),
    /// Accessed attribute isn't in the map storage.
    #[error("unknown attribute: {0}")]
    UnknownAttribute(&'static str),
}

impl From<StmError> for CMapError {
    fn from(value: StmError) -> Self {
        Self::FailedTransaction(value)
    }
}

// by default, a map error inside of a transaction will result in a retry
impl From<CMapError> for StmError {
    fn from(value: CMapError) -> Self {
        match value {
            CMapError::FailedTransaction(e) => e,
            CMapError::FailedAttributeMerge(_)
            | CMapError::FailedAttributeSplit(_)
            | CMapError::IncorrectGeometry(_)
            | CMapError::MissingAttribute(_)
            | CMapError::UnknownAttribute(_) => StmError::Retry,
        }
    }
}
