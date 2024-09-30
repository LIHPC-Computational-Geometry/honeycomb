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

use crate::triangulation::TriangulateError::UndefinedFace;
pub use ear_clipping::process_cell as earclip_cell;
pub use fan::process_cell as fan_cell;
pub use fan::process_convex_cell as fan_convex_cell;
use honeycomb_core::cmap::{CMap2, DartIdentifier, FaceIdentifier};
use honeycomb_core::geometry::{CoordsFloat, Vertex2};
// ------ CONTENT

pub enum TriangulateError {
    AlreadyTriangulated,
    NoEar,
    NonFannable,
    NotEnoughDarts(usize),
    TooManyDarts(usize),
    UndefinedFace(String),
}

#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_abs_to_unsigned
)]
pub fn check_requirements(
    n_darts_face: usize,
    n_darts_allocated: usize,
    face_id: FaceIdentifier,
) -> Result<(), TriangulateError> {
    match n_darts_face {
        1 | 2 => {
            return Err(TriangulateError::UndefinedFace(format!(
                "face {face_id} has less than three vertices"
            )));
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
    darts: &[DartIdentifier],
    face_id: FaceIdentifier,
) -> Result<Vec<Vertex2<T>>, TriangulateError> {
    let tmp = darts
        .iter()
        .map(|dart_id| cmap.vertex(cmap.vertex_id(*dart_id)));
    if tmp.clone().any(|v| v.is_none()) {
        Err(UndefinedFace(format!(
            "face {face_id} has one or more undefined vertices"
        )))
    } else {
        Ok(tmp.map(Option::unwrap).collect()) // safe unwrap due to if
    }
}

// ------ TESTS

#[cfg(test)]
mod tests;
