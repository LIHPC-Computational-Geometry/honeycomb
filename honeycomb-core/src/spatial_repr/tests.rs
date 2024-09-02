// ------ IMPORTS

// ------ CONTENT

use super::CoordsFloat;

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

macro_rules! almost_equals {
    // Coords
    ($lhs: expr, $rhs: expr) => {
        almost_equal($lhs.x, $rhs.x) & almost_equal($lhs.x, $rhs.x)
    };
    // Vector / Vertex
    (($lhs: expr, $rhs: expr)) => {
        almost_equal($lhs.x(), $rhs.x()) & almost_equal($lhs.y(), $rhs.y())
    };
}

// --- coords

// --- vector

mod vector {
    use super::almost_equal;
    use crate::{CoordsError, Vector2};
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
                assert!(almost_equals!((
                    along_x.unit_dir().unwrap(),
                    Vector2::<$t>::unit_x()
                )));
                assert!(almost_equals!((
                    Vector2::<$t>::unit_x().unit_dir().unwrap(),
                    Vector2::<$t>::unit_x()
                )));
                assert!(almost_equals!((
                    along_y.unit_dir().unwrap(),
                    Vector2::<$t>::unit_y()
                )));
                assert!(almost_equals!((
                    (along_x + along_y).unit_dir().unwrap(),
                    Vector2::<$t>(4.0 / 5.0, 3.0 / 5.0)
                )));
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
                assert!(almost_equals!((
                    along_x.normal_dir().unwrap(),
                    Vector2::<$t>::unit_y()
                )));
                assert!(almost_equals!((
                    Vector2::<$t>::unit_x().normal_dir().unwrap(),
                    Vector2::<$t>::unit_y()
                )));
                assert!(almost_equals!((
                    along_y.normal_dir().unwrap(),
                    -Vector2::<$t>::unit_x()
                )));
                assert!(almost_equals!((
                    Vector2::<$t>::unit_y().normal_dir().unwrap(),
                    -Vector2::<$t>::unit_x()
                )));
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

mod vertex {
    use crate::{Vector2, Vertex2};
    // tests
    #[test]
    fn add_vertex_vector() {
        {
            let mut a: Vertex2<f64> = Vertex2(1.0, 1.0);
            let b: Vector2<f64> = Vector2(1.0, 0.0);
            let a_moved = a + b;
            assert_eq!(a_moved, Vertex2(2.0, 1.0));
            a += &b;
            assert_eq!(a, a_moved);
            a += b;
            assert_eq!(a, Vertex2(3.0, 1.0));
        }
        {
            let mut a: Vertex2<f32> = Vertex2(1.0, 1.0);
            let b: Vector2<f32> = Vector2(1.0, 0.0);
            let a_moved = a + b;
            assert_eq!(a_moved, Vertex2(2.0, 1.0));
            a += &b;
            assert_eq!(a, a_moved);
            a += b;
            assert_eq!(a, Vertex2(3.0, 1.0));
        }
    }

    #[test]
    fn sub_vertex_vector() {
        {
            let mut a: Vertex2<f64> = Vertex2(1.0, 1.0);
            let b: Vector2<f64> = Vector2(1.0, 0.0);
            let a_moved = a - b;
            assert_eq!(a_moved, Vertex2(0.0, 1.0));
            a -= &b;
            assert_eq!(a, a_moved);
            a -= b;
            assert_eq!(a, Vertex2(-1.0, 1.0));
        }
        {
            let mut a: Vertex2<f32> = Vertex2(1.0, 1.0);
            let b: Vector2<f32> = Vector2(1.0, 0.0);
            let a_moved = a - b;
            assert_eq!(a_moved, Vertex2(0.0, 1.0));
            a -= &b;
            assert_eq!(a, a_moved);
            a -= b;
            assert_eq!(a, Vertex2(-1.0, 1.0));
        }
    }

    #[test]
    fn sub_vertex_vertex() {
        {
            let a: Vertex2<f64> = Vertex2(1.0, 1.0);
            let b: Vertex2<f64> = Vertex2(1.0, 0.0);
            let ab = b - a;
            assert_eq!(ab, Vector2(0.0, -1.0));
        }
        {
            let a: Vertex2<f32> = Vertex2(1.0, 1.0);
            let b: Vertex2<f32> = Vertex2(1.0, 0.0);
            let ab = b - a;
            assert_eq!(ab, Vector2(0.0, -1.0));
        }
    }
}
