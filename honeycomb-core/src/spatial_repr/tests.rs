// ------ IMPORTS

// ------ CONTENT

use crate::CoordsFloat;

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
        abs_diff / abs_sum.min(T::max_value()) < T::epsilon()
    }
}

// --- coords

mod coords {
    // utils
    use super::almost_equal;
    use crate::Coords2;
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

mod vector {
    // utils
    use super::almost_equal;
    use crate::{CoordsError, Vector2};
    macro_rules! almost_equal_vec {
        ($lhs: expr, $rhs: expr, $t: ty) => {
            almost_equal($lhs.x(), $rhs.x()) & almost_equal($lhs.y(), $rhs.y())
        };
    }
    // tests
    macro_rules! generate_dot_prod_test {
        ($id: ident, $t: ty) => {
            #[test]
            fn $id() {
                let along_x = Vector2::<$t>::unit_x() * 15.0;
                let along_y = Vector2::<$t>::unit_y() * 10.0;
                assert!(almost_equal(along_x.dot(&along_y), 0.0));
                assert!(almost_equal(along_x.dot(&Vector2::unit_x()), 15.0));
                assert!(almost_equal(along_y.dot(&Vector2::unit_y()), 10.0));
            }
        };
    }
    macro_rules! generate_unit_dir_test {
        ($id: ident, $t: ty) => {
            #[test]
            fn $id() {
                let along_x = Vector2::<$t>::unit_x() * 4.0;
                let along_y = Vector2::<$t>::unit_y() * 3.0;
                assert!(almost_equal_vec!(
                    along_x.unit_dir().unwrap(),
                    Vector2::<$t>::unit_x(),
                    $t
                ));
                assert!(almost_equal_vec!(
                    Vector2::<$t>::unit_x().unit_dir().unwrap(),
                    Vector2::<$t>::unit_x(),
                    $t
                ));
                assert!(almost_equal_vec!(
                    along_y.unit_dir().unwrap(),
                    Vector2::<$t>::unit_y(),
                    $t
                ));
                assert!(almost_equal_vec!(
                    (along_x + along_y).unit_dir().unwrap(),
                    Vector2::<$t>::from((4.0 / 5.0, 3.0 / 5.0)),
                    $t
                ));
                let origin: Vector2<$t> = Vector2::default();
                assert_eq!(origin.unit_dir(), Err(CoordsError::InvalidUnitDir));
            }
        };
    }
    macro_rules! generate_normal_dir_test {
        ($id: ident, $t: ty) => {
            #[test]
            fn $id() {
                let along_x = Vector2::<$t>::unit_x() * 4.0;
                let along_y = Vector2::<$t>::unit_y() * 3.0;
                assert!(almost_equal_vec!(
                    along_x.normal_dir().unwrap(),
                    Vector2::<$t>::unit_y(),
                    $t
                ));
                assert!(almost_equal_vec!(
                    Vector2::<$t>::unit_x().normal_dir().unwrap(),
                    Vector2::<$t>::unit_y(),
                    $t
                ));
                assert!(almost_equal_vec!(
                    along_y.normal_dir().unwrap(),
                    -Vector2::<$t>::unit_x(),
                    $t
                ));
                assert!(almost_equal_vec!(
                    Vector2::<$t>::unit_y().normal_dir().unwrap(),
                    -Vector2::<$t>::unit_x(),
                    $t
                ));
                let origin: Vector2<$t> = Vector2::default();
                assert_eq!(origin.normal_dir(), Err(CoordsError::InvalidUnitDir));
            }
        };
    }
    // generation
    generate_dot_prod_test!(dot_product_simple, f32);
    generate_dot_prod_test!(dot_product_double, f64);

    generate_unit_dir_test!(unit_dir_simple, f32);
    generate_unit_dir_test!(unit_dir_double, f64);

    generate_normal_dir_test!(normal_dir_simple, f32);
    generate_normal_dir_test!(normal_dir_double, f64);
}

// --- vertex

mod vertex {}
