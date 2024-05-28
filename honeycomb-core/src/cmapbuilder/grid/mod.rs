//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

mod building_routines;
mod descriptor;

// ------ RE-EXPORTS

pub(super) use building_routines::{build2_grid, build2_splitgrid};
pub use descriptor::GridDescriptor;

// ------ CONTENT

// ------ TESTS
#[cfg(test)]
mod tests;
