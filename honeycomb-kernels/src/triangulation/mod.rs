//! Polygon triangulation functions
//!
//! This module contains implementations of simple polygon triangulation methods. These are not
//! meshing functions; our goal with these is to cut existing cells of an irregular mesh into
//! triangular cells.
//!
//! With consideration to the above, we implement two polygon triangulation methods:
//! - fanning -- two versions of this are implemented:
//!     - a defensive one where the function actively search for a valid vertex to fan from
//!     - a specific one which assume the cell is convex; it fans the polygon from its first vertex
//! - ear clipping -- this method isn't algorithmically efficient, but (a) we operate on small
//!   cells, and (b) it covers our needs (non-fannable polygons without holes)

// ------ MODULE DECLARATIONS

mod ear_clipping;
mod fan;

// ------ PUBLIC RE-EXPORTS

pub use ear_clipping::process_cell as earclip_cell;
pub use fan::process_cell as fan_cell;
pub use fan::process_convex_cell as fan_convex_cell;

// ------ TESTS

#[cfg(test)]
mod tests;
