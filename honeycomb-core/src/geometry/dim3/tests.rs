// ------ IMPORTS

use super::super::{CoordsFloat, Vector3, Vertex3};

// ------ CONTENT

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
    ($lhs:expr, $rhs:expr) => {
        almost_equal($lhs.x, $rhs.x) & almost_equal($lhs.y, $rhs.y) & almost_equal($lhs.z, $rhs.z)
    };
    // Vector / Vertex
    (($lhs:expr, $rhs:expr)) => {
        almost_equal($lhs.x(), $rhs.x())
            & almost_equal($lhs.y(), $rhs.y())
            & almost_equal($lhs.z(), $rhs.z())
    };
}

// --- coords
// --- vector
mod vector {
    use super::*;
    use crate::prelude::CoordsError;

    // tests
    macro_rules! generate_dot_prod_test {
        ($id:ident, $t:ty) => {
            #[allow(clippy::float_cmp)]
            #[test]
            fn $id() {
                let mut along_x = Vector3::<$t>::unit_x() * 15.0;
                let mut along_y = Vector3::<$t>::unit_y() * 10.0;
                let mut along_z = Vector3::<$t>::unit_z() * 5.0;
                assert_eq!(along_x.dot(&along_y), 0.0);
                assert_eq!(along_x.dot(&along_z), 0.0);
                assert_eq!(along_y.dot(&along_z), 0.0);
                along_x /= 15.0;
                along_y *= 10.0;
                along_z -= Vector3::<$t>::unit_z();
                assert!(almost_equal(along_x.dot(&Vector3::unit_x()), 1.0));
                assert!(almost_equal(along_y.dot(&Vector3::unit_y()), 100.0));
                assert!(almost_equal(along_z.dot(&Vector3::unit_z()), 4.0));
            }
        };
    }

    macro_rules! generate_unit_dir_test {
        ($id:ident, $t:ty) => {
            #[test]
            fn $id() {
                let along_x = Vector3::<$t>::unit_x() * 4.0;
                let along_y = Vector3::<$t>::unit_y() * 3.0;
                let along_z = Vector3::<$t>::unit_z() * 2.0;
                assert!(almost_equals!((
                    along_x.unit_dir().unwrap(),
                    Vector3::<$t>::unit_x()
                )));
                assert!(almost_equals!((
                    Vector3::<$t>::unit_x().unit_dir().unwrap(),
                    Vector3::<$t>::unit_x()
                )));
                assert!(almost_equals!((
                    along_y.unit_dir().unwrap(),
                    Vector3::<$t>::unit_y()
                )));
                assert!(almost_equals!((
                    along_z.unit_dir().unwrap(),
                    Vector3::<$t>::unit_z()
                )));
                assert!(almost_equals!((
                    (along_x + along_y + along_z).unit_dir().unwrap(),
                    Vector3::<$t>(
                        (16.0 / 29.0 as $t).sqrt(),
                        (9.0 / 29.0 as $t).sqrt(),
                        (4.0 / 29.0 as $t).sqrt()
                    )
                )));
                let origin: Vector3<$t> = Vector3::default();
                assert_eq!(origin.unit_dir(), Err(CoordsError::InvalidUnitDir));
            }
        };
    }

    macro_rules! generate_cross_product_test {
        ($id:ident, $t:ty) => {
            #[test]
            fn $id() {
                let v1 = Vector3::<$t>(1.0, 2.0, 3.0);
                let v2 = Vector3::<$t>(4.0, 5.0, 6.0);
                let cross_product = v1.cross(&v2);
                assert!(almost_equals!((
                    cross_product,
                    Vector3::<$t>(-3.0, 6.0, -3.0)
                )));
                assert!(almost_equals!((
                    Vector3::<$t>::unit_x().cross(&Vector3::<$t>::unit_y()),
                    Vector3::<$t>::unit_z()
                )));
            }
        };
    }

    // generation
    generate_dot_prod_test!(dot_product_simple, f32);
    generate_dot_prod_test!(dot_product_double, f64);
    generate_unit_dir_test!(unit_dir_simple, f32);
    generate_unit_dir_test!(unit_dir_double, f64);
    generate_cross_product_test!(cross_product_simple, f32);
    generate_cross_product_test!(cross_product_double, f64);

    #[test]
    fn test_into_inner() {
        let v = Vector3::from((1.0_f32, 2.0, 3.0));
        let (x, y, z) = v.into_inner();
        almost_equal(x, 1.0);
        almost_equal(y, 2.0);
        almost_equal(z, 3.0);
    }
}

// --- vertex
mod vertex {
    use crate::geometry::dim3::tests::almost_equal;

    use super::{Vector3, Vertex3};

    #[test]
    fn test_getters() {
        let vertex = Vertex3::from((1.0, 2.0, 3.0));
        almost_equal(vertex.x(), 1.0);
        almost_equal(vertex.y(), 2.0);
        almost_equal(vertex.z(), 3.0);
    }

    #[test]
    fn test_into_inner() {
        let vertex = Vertex3(1.0_f32, 2.0, 3.0);
        let (x, y, z) = vertex.into_inner();
        almost_equal(x, 1.0);
        almost_equal(y, 2.0);
        almost_equal(z, 3.0);
    }

    #[test]
    fn test_average() {
        let v1 = Vertex3(0.0, 0.0, 0.0);
        let v2 = Vertex3(2.0, 4.0, 6.0);
        let avg = Vertex3::average(&v1, &v2);
        assert_eq!(avg, Vertex3(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_average_with_negative_values() {
        let v1 = Vertex3(-1.0, -2.0, -3.0);
        let v2 = Vertex3(1.0, 2.0, 3.0);
        let avg = Vertex3::average(&v1, &v2);
        assert_eq!(avg, Vertex3(0.0_f32, 0.0_f32, 0.0_f32));
    }

    #[test]
    fn test_average_with_different_types() {
        let v1 = Vertex3(0.0, 1.0, 2.0);
        let v2 = Vertex3(2.0, 3.0, 4.0);
        let avg = Vertex3::average(&v1, &v2);
        assert_eq!(avg, Vertex3(1.0_f32, 2.0_f32, 3.0_f32));
    }

    // tests
    #[test]
    fn add_vertex_vector() {
        let mut a: Vertex3<f64> = Vertex3(1.0, 1.0, 1.0);
        let b: Vector3<f64> = Vector3(1.0, 0.0, 0.0);
        let a_moved = a + b;
        assert_eq!(a_moved, Vertex3(2.0, 1.0, 1.0));
        a += &b;
        assert_eq!(a, a_moved);
        a += b;
        assert_eq!(a, Vertex3(3.0, 1.0, 1.0));
    }

    #[test]
    fn sub_vertex_vector() {
        let mut a: Vertex3<f64> = Vertex3(1.0, 1.0, 1.0);
        let b: Vector3<f64> = Vector3(1.0, 0.0, 0.0);
        let a_moved = a - b;
        assert_eq!(a_moved, Vertex3(0.0, 1.0, 1.0));
        a -= &b;
        assert_eq!(a, a_moved);
        a -= b;
        assert_eq!(a, Vertex3(-1.0, 1.0, 1.0));
    }

    #[test]
    fn sub_vertex_vertex() {
        let a: Vertex3<f64> = Vertex3(1.0, 1.0, 1.0);
        let b: Vertex3<f64> = Vertex3(1.0, 0.0, 2.0);
        let ab = b - a;
        assert!(almost_equal(ab.x(), 0.0));
        assert!(almost_equal(ab.y(), -1.0));
        assert!(almost_equal(ab.z(), 1.0));
    }
}
