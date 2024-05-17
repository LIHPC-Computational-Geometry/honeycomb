// ------ IMPORTS

// ------ CONTENT

use crate::{Coords2, CoordsFloat};

// --- common

// scalar equality function
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

mod coords {
    // utils
    use super::*;
    macro_rules! almost_equal_coords {
        ($lhs: expr, $rhs: expr) => {
            almost_equal($lhs.x, $rhs.x) & almost_equal($lhs.x, $rhs.x)
        };
    }
    // tests
    macro_rules! generate_sum_test {
        ($id: ident, $t: ty) => {
            #[test]
            fn $id() {
                let collection = [
                    Coords2::unit_x(),
                    Coords2::unit_x(),
                    Coords2::unit_x(),
                    Coords2::unit_y(),
                    Coords2::unit_y(),
                    Coords2::unit_y(),
                ];

                let owned_sum: Coords2<$t> = collection.into_iter().sum();
                let borrowed_sum: Coords2<$t> = collection.iter().sum();
                let ref_value: Coords2<$t> = Coords2::from((3.0, 3.0));
                assert!(almost_equal_coords!(owned_sum, ref_value));
                assert!(almost_equal_coords!(borrowed_sum, ref_value));
            }
        };
    }
    // generation
    generate_sum_test!(sum_simple, f32);
    generate_sum_test!(sum_double, f64);
}

// --- vector

mod vector {}

// --- vertex

mod vertex {}
