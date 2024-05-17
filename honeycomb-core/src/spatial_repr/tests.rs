// ------ IMPORTS

// ------ CONTENT

use crate::CoordsFloat;

// --- common

// ref: https://floating-point-gui.de/errors/comparison/
fn almost_equal<T: CoordsFloat>(a: T, b: T) -> bool {
    let abs_diff = (a - b).abs();
    let abs_sum = a.abs() + b.abs();
    if a == b {
        // early return
        true
    } else if a.is_zero() | b.is_zero() | (abs_sum < T::min_positive_value()) {
        // close to zero
        abs_diff < (T::epsilon() * T::min_positive_value())
    } else {
        // regular case
        abs_diff / (abs_sum).min(T::max_value()) < T::epsilon()
    }
}

// --- coords

mod coords {}

// --- vector

mod vector {}

// --- vertex

mod vertex {}
