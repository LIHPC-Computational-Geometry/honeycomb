use crate::TwoMap;

/// Generate a [TwoMap] representing a mesh made up of squares.
///
/// This function builds and returns a 2-map representing a square mesh
/// made of `n_square * n_square` square cells.
///
/// # Arguments
///
/// - `n_square: usize` -- Dimension of the returned mesh.
///
/// ## Generics
///
/// - `const N_MARKS: usize` -- Generic parameter of the returned [TwoMap]
///
/// # Return / Panic
///
/// Returns a boundary-less [TwoMap] of the specified size.
///
/// # Example
///
/// ```text
///
/// ```
///
pub fn square_two_map<const N_MARKS: usize>(n_square: usize) -> TwoMap<N_MARKS> {
    todo!()
}
