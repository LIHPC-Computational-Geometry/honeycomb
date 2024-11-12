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

// ------ CONTENT

use honeycomb_core::cmap::{CMap2, DartId};
use honeycomb_core::geometry::{CoordsFloat, Vertex2};
use thiserror::Error;

/// Error-modeling enum for triangulation routines.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum TriangulateError {
    /// The face to triangulate is already a triangle.
    #[error("face is already a triangle")]
    AlreadyTriangulated,
    /// The face has no ear to use for triangulation using the ear clipping method.
    #[error("no ear found in the polygon to triangulate")]
    NoEar,
    /// The face is not fannable, i.e. there is no "star" vertex.
    #[error("no star in the polygon to triangulate")]
    NonFannable,
    /// The number of darts passed to create the new segments is too low. The `usize` value
    /// is the number of missing darts.
    #[error("not enough darts were passed to the triangulation function - missing `{0}`")]
    NotEnoughDarts(usize),
    /// The number of darts passed to create the new segments is too high. The `usize` value
    /// is the number of excess darts.
    #[error("too many darts were passed to the triangulation function - missing `{0}`")]
    TooManyDarts(usize),
    /// The face is not fit for triangulation. The `String` contains information about the reason.
    #[error("face isn't defined correctly - {0}")]
    UndefinedFace(&'static str),
}

#[allow(clippy::missing_errors_doc)]
/// Checks if a face meets the requirements for triangulation.
///
/// This function performs several checks on a face before it can be triangulated:
/// 1. Ensures the face has at least 3 vertices.
/// 2. Verifies that the face is not already triangulated.
/// 3. Confirms that the correct number of darts have been allocated for triangulation.
///
/// # Arguments
///
/// * `n_darts_face` - The number of darts in the face to be triangulated.
/// * `n_darts_allocated` - The number of darts allocated for the triangulation process.
/// * `face_id` - The identifier of the face being checked.
///
/// # Return / Errors
///
/// This function can return:
/// - `Ok(())` if all checks pass and the face is ready for triangulation.
/// - `Err(TriangulateError)` if any check fails, with a specific error describing the issue.
///
/// A failed check can result in the following errors:
/// - `TriangulateError::UndefinedFace` - If the face has fewer than 3 vertices.
/// - `TriangulateError::AlreadyTriangulated` - If the face already has exactly 3 vertices.
/// - `TriangulateError::NotEnoughDarts` - If there aren't enough darts allocated for triangulation.
/// - `TriangulateError::TooManyDarts` - If there are too many darts allocated for triangulation.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_abs_to_unsigned
)]
pub fn check_requirements(
    n_darts_face: usize,
    n_darts_allocated: usize,
) -> Result<(), TriangulateError> {
    match n_darts_face {
        1 | 2 => {
            return Err(TriangulateError::UndefinedFace("less than 3 vertices"));
        }
        3 => {
            return Err(TriangulateError::AlreadyTriangulated);
        }
        _ => {}
    }

    // check the value of n_allocated - n_expected
    match n_darts_allocated as isize - (n_darts_face as isize - 3) * 2 {
        diff @ ..0 => {
            return Err(TriangulateError::NotEnoughDarts(diff.abs() as usize));
        }
        0 => {}
        diff @ 1.. => {
            return Err(TriangulateError::TooManyDarts(diff as usize));
        }
    }

    Ok(())
}

fn fetch_face_vertices<T: CoordsFloat>(
    cmap: &CMap2<T>,
    darts: &[DartId],
) -> Result<Vec<Vertex2<T>>, TriangulateError> {
    let tmp = darts
        .iter()
        .map(|dart_id| cmap.vertex(cmap.vertex_id(*dart_id)));
    if tmp.clone().any(|v| v.is_none()) {
        Err(TriangulateError::UndefinedFace(
            "one or more undefined vertices",
        ))
    } else {
        Ok(tmp.map(Option::unwrap).collect()) // safe unwrap due to if
    }
}

/// Compute the cross product: `v1v2 x v2v3`.
fn crossp_from_verts<T: CoordsFloat>(v1: &Vertex2<T>, v2: &Vertex2<T>, v3: &Vertex2<T>) -> T {
    (v2.x() - v1.x()) * (v3.y() - v2.y()) - (v2.y() - v1.y()) * (v3.x() - v2.x())
}

// ------ TESTS

#[cfg(test)]
mod tests;
