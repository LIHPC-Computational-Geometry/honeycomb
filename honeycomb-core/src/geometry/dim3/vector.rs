use crate::prelude::{CoordsError, CoordsFloat, Vector2};

/// # 3D vector structure
///
/// ## Generics
///
/// - `T: CoordsFloat` -- Generic FP type for coordinates.
///
/// ## Example
///
/// ```
/// # use honeycomb_core::prelude::CoordsError;
/// # fn main() -> Result<(), CoordsError> {
/// use honeycomb_core::geometry::Vector3;
///
/// let unit_x = Vector3::unit_x();
/// let unit_y = Vector3::unit_y();
/// let unit_z = Vector3::unit_z();
///
/// assert_eq!(unit_x.dot(&unit_y), 0.0);
/// assert_eq!(unit_x.cross(&unit_y), unit_z);
///
/// let sum: Vector3<f64> = unit_x + unit_y + unit_z;
///
/// assert_eq!(sum.norm(), 3.0_f64.sqrt());
/// assert_eq!(
///     sum.unit_dir()?,
///     Vector3(
///         1.0 / 3.0_f64.sqrt(),
///         1.0 / 3.0_f64.sqrt(),
///         1.0 / 3.0_f64.sqrt(),
///     ),
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector3<T: CoordsFloat>(pub T, pub T, pub T);

unsafe impl<T: CoordsFloat> Send for Vector3<T> {}
unsafe impl<T: CoordsFloat> Sync for Vector3<T> {}

impl<T: CoordsFloat> Vector3<T> {
    /// Return a unit vector along the `x` axis.
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub fn unit_x() -> Self {
        Self(T::one(), T::zero(), T::zero())
    }

    /// Return a unit vector along the `y` axis.
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub fn unit_y() -> Self {
        Self(T::zero(), T::one(), T::zero())
    }

    /// Return a unit vector along the `z` axis.
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub fn unit_z() -> Self {
        Self(T::zero(), T::zero(), T::one())
    }

    /// Consume `self` to return inner values.
    pub fn into_inner(self) -> (T, T, T) {
        (self.0, self.1, self.2)
    }

    /// Return the value of the `x` coordinate of the vector.
    pub fn x(&self) -> T {
        self.0
    }

    /// Return the value of the `y` coordinate of the vector.
    pub fn y(&self) -> T {
        self.1
    }

    /// Return the value of the `z` coordinate of the vector.
    pub fn z(&self) -> T {
        self.2
    }

    /// Compute the norm of `self`.
    pub fn norm(&self) -> T {
        (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
    }

    /// Compute the direction of `self` as a unit vector.
    ///
    /// # Errors
    ///
    /// This method will return an error if called on a null `Vector3`.
    pub fn unit_dir(&self) -> Result<Self, CoordsError> {
        let norm = self.norm();
        if norm.is_zero() {
            Err(CoordsError::InvalidUnitDir)
        } else {
            Ok(*self / norm)
        }
    }
    ///Return the dot product between `self` and `other`.
    pub fn dot(&self, other: &Vector3<T>) -> T {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    /// Return the cross product between `self` and `other`.
    #[must_use = "result is not used, consider removing this method call"]
    pub fn cross(&self, other: &Vector3<T>) -> Self {
        Self(
            self.1 * other.2 - self.2 * other.1,
            self.2 * other.0 - self.0 * other.2,
            self.0 * other.1 - self.1 * other.0,
        )
    }
}

// Building trait

impl<T: CoordsFloat> From<(T, T, T)> for Vector3<T> {
    fn from((x, y, z): (T, T, T)) -> Self {
        Self(x, y, z)
    }
}

impl<T: CoordsFloat> From<Vector2<T>> for Vector3<T> {
    fn from(v: Vector2<T>) -> Self {
        Self(v.0, v.1, T::zero())
    }
}

// Basic operations

impl<T: CoordsFloat> std::ops::Add<Vector3<T>> for Vector3<T> {
    type Output = Self;
    fn add(self, rhs: Vector3<T>) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<Vector3<T>> for Vector3<T> {
    fn add_assign(&mut self, rhs: Vector3<T>) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl<T: CoordsFloat> std::ops::Sub<Vector3<T>> for Vector3<T> {
    type Output = Self;
    fn sub(self, rhs: Vector3<T>) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<Vector3<T>> for Vector3<T> {
    fn sub_assign(&mut self, rhs: Vector3<T>) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
        self.2 -= rhs.2;
    }
}

impl<T: CoordsFloat> std::ops::Mul<T> for Vector3<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl<T: CoordsFloat> std::ops::MulAssign<T> for Vector3<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs;
        self.1 *= rhs;
        self.2 *= rhs;
    }
}

impl<T: CoordsFloat> std::ops::Div<T> for Vector3<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        assert!(!rhs.is_zero());
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl<T: CoordsFloat> std::ops::DivAssign<T> for Vector3<T> {
    fn div_assign(&mut self, rhs: T) {
        assert!(!rhs.is_zero());
        self.0 /= rhs;
        self.1 /= rhs;
        self.2 /= rhs;
    }
}

impl<T: CoordsFloat> std::ops::Neg for Vector3<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, -self.2)
    }
}
