//! parallel-friendly combinatorial maps implementations

mod common;
mod dim2;

pub use common::{is_null, PDartIdentifier};
pub use dim2::{orbits::POrbit2, structure::PMap2};
