//! Geometric data embedding
//!
//! Note:
//! - The initialization of geometric data should eventually be done by
//!   the combinatorial map constructor. Until then, the user has to
//!   manually initialize it through this module's tools.
//! - A custom vector type is needed
//!
//! This module contains all code used to handle geometric data
//! embedding. This includes ID types, spatial representation,
//! (un)sewing policies.

// ------ MODULE DECLARATIONS

// ------ IMPORTS

#[cfg(doc)]
use crate::CMap2;

// ------ CONTENT

/// Geometrical policy of the sewing operation.
///
/// All sewing operation include two darts, which we refer to as
/// *lhs_dart* and *rhs_dart*, hence the naming conventions.
///
/// For more information on why a strict policy definition is needed,
/// refer to the [user guide][UG].
///
/// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
///
/// # Example
///
/// Refer to the user guide and [CMap2] examples.
///
#[derive(Debug, Default)]
pub enum SewPolicy {
    /// Default policy.
    ///
    /// *lhs_dart* embedded data is used to overwrite *rhs_dart*
    /// embedded data, effectively stretching *rhs_dart* to
    /// *lhs_dart*'s position.
    #[default]
    StretchLeft,
    /// *rhs_dart* embedded data is used to overwrite *lhs_dart*
    /// embedded data, effectively stretching *lhs_dart* to
    /// *rhs_dart*'s position.
    StretchRight,
    /// *lhs_dart* & *rhs_dart* embedded data are both used to
    /// compute a "middle ground" used to overwrite both initial
    /// positions.
    StretchAverage,
}

/// Geometrical policy of the sewing operation.
///
/// All unsewing operation include two darts, which we refer to as
/// *lhs_dart* and *rhs_dart*, hence the naming conventions. Note that
/// the second dart is not necessarily specified as it can be computed
/// the index of the beta function affected by the operation.
///
/// For more information on why a strict policy definition is needed,
/// refer to the [user guide][UG].
///
/// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/
///
/// # Example
///
/// Refer to the user guide and [CMap2] examples.
///
#[derive(Debug, Default)]
pub enum UnsewPolicy {
    /// Default policy.
    ///
    /// Vertices associated with the unsewed darts are duplicated in order
    /// to allow for a shift in coordinates without mutual stretching.
    #[default]
    Duplicate,
    DoNothing,
}
