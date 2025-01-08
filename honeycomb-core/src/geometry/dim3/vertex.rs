//! Custom spatial representation
//!
//! This module contains all code used to model vertices.

use super::super::Vector3;
use crate::prelude::{AttributeBind, AttributeUpdate, OrbitPolicy, Vertex2, VertexIdType};
use crate::{attributes::AttrSparseVec, geometry::CoordsFloat};

/// # 2D vertex structure
///
/// ## Attribute behavior
///
/// - binds to 0-cells,
/// - merge policy: the new vertex is placed at the midpoint between the two existing ones,
/// - split policy: the current vertex is duplicated,
/// - fallback policies: default implementations are used.
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
/// use honeycomb_core::geometry::{Vector3, Vertex3};
///
/// let v1 = Vertex3(1.0, 0.0, 0.0);
/// let v2 = Vertex3(1.0, 1.0, 1.0);
///
/// assert_eq!(v1.x(), 1.0);
/// assert_eq!(v1.y(), 0.0);
/// assert_eq!(v1.z(), 0.0);
///
/// let two: f64 = 2.0;
/// // vectorAB = vertexB - vertexA
/// let v2_minus_v1: Vector3<f64> = v2 - v1;
///
/// assert_eq!(v2_minus_v1.norm(), two.sqrt());
/// assert_eq!(v2_minus_v1.unit_dir()?, Vector3(0.0, 1.0 / two.sqrt(), 1.0 / two.sqrt()));
///
/// let mut v3 = Vertex3(0.0, 1.0, 1.0);
/// // vertexA + vectorB = vertexA'
/// v3 += v2_minus_v1;
///
/// assert_eq!(v3.x(), 0.0);
/// assert_eq!(v3.y(), 2.0);
/// assert_eq!(v3.z(), 2.0);
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vertex3<T: CoordsFloat>(pub T, pub T, pub T);

unsafe impl<T: CoordsFloat> Send for Vertex3<T> {}
unsafe impl<T: CoordsFloat> Sync for Vertex3<T> {}

impl<T: CoordsFloat> Vertex3<T> {
    /// Consume `self` to return inner values.
    pub fn into_inner(self) -> (T, T, T) {
        (self.0, self.1, self.2)
    }

    /// Return the value of the `x` coordinate of the vertex.
    pub fn x(&self) -> T {
        self.0
    }

    /// Return the value of the `y` coordinate of the vertex.
    pub fn y(&self) -> T {
        self.1
    }

    /// Return the value of the `z` coordinate of the vertex.
    pub fn z(&self) -> T {
        self.2
    }

    /// Compute the mid-point between two vertices.
    ///
    /// # Panics
    ///
    /// This function may panic if it cannot initialize an object `T: CoordsFloat` from the value
    /// `2.0`. The chance of this happening when using `T = f64` or `T = f32` is most likely zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// use honeycomb_core::geometry::Vertex3;
    ///
    /// let far_far_away: Vertex3<f64> = Vertex3(2.0, 2.0, 2.0);
    /// let origin: Vertex3<f64> = Vertex3::default();
    ///
    /// assert_eq!(Vertex3::average(&origin, &far_far_away), Vertex3(1.0, 1.0, 1.0));
    /// ```
    pub fn average(lhs: &Vertex3<T>, rhs: &Vertex3<T>) -> Vertex3<T> {
        let two = T::from(2.0).unwrap();
        Vertex3(
            (lhs.0 + rhs.0) / two,
            (lhs.1 + rhs.1) / two,
            (lhs.2 + rhs.2) / two,
        )
    }
}

// Building trait

impl<T: CoordsFloat> From<(T, T, T)> for Vertex3<T> {
    fn from((x, y, z): (T, T, T)) -> Self {
        Self(x, y, z)
    }
}

impl<T: CoordsFloat> From<Vertex2<T>> for Vertex3<T> {
    fn from(v: Vertex2<T>) -> Self {
        Self(v.0, v.1, T::zero())
    }
}

// Basic operations

// -- add flavors

impl<T: CoordsFloat> std::ops::Add<Vector3<T>> for Vertex3<T> {
    // Vertex + Vector = Vertex
    type Output = Self;
    fn add(self, rhs: Vector3<T>) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<Vector3<T>> for Vertex3<T> {
    fn add_assign(&mut self, rhs: Vector3<T>) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl<T: CoordsFloat> std::ops::Add<&Vector3<T>> for Vertex3<T> {
    // Vertex + Vector = Vertex
    type Output = Self;
    fn add(self, rhs: &Vector3<T>) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<&Vector3<T>> for Vertex3<T> {
    fn add_assign(&mut self, rhs: &Vector3<T>) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

// -- sub flavors
impl<T: CoordsFloat> std::ops::Sub<Vector3<T>> for Vertex3<T> {
    // Vertex - Vector = Vertex
    type Output = Self;
    fn sub(self, rhs: Vector3<T>) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<Vector3<T>> for Vertex3<T> {
    fn sub_assign(&mut self, rhs: Vector3<T>) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
        self.2 -= rhs.2;
    }
}

impl<T: CoordsFloat> std::ops::Sub<&Vector3<T>> for Vertex3<T> {
    // Vertex - Vector = Vertex
    type Output = Self;
    fn sub(self, rhs: &Vector3<T>) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<&Vector3<T>> for Vertex3<T> {
    fn sub_assign(&mut self, rhs: &Vector3<T>) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
        self.2 -= rhs.2;
    }
}

impl<T: CoordsFloat> std::ops::Sub<Vertex3<T>> for Vertex3<T> {
    type Output = Vector3<T>;
    fn sub(self, rhs: Vertex3<T>) -> Self::Output {
        Vector3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: CoordsFloat> AttributeUpdate for Vertex3<T> {
    fn merge(attr1: Self, attr2: Self) -> Self {
        Self::average(&attr1, &attr2)
    }

    fn split(attr: Self) -> (Self, Self) {
        (attr, attr)
    }
}

impl<T: CoordsFloat> AttributeBind for Vertex3<T> {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}
