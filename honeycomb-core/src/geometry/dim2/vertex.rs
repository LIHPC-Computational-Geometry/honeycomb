//! Custom spatial representation
//!
//! This module contains all code used to model vertices.

use crate::attributes::{AttrSparseVec, AttributeBind, AttributeError, AttributeUpdate};
use crate::cmap::{OrbitPolicy, VertexIdType};
use crate::geometry::{CoordsFloat, Vector2};

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
/// use honeycomb_core::prelude::{Vector2, Vertex2};
///
/// let v1 = Vertex2(1.0, 0.0);
/// let v2 = Vertex2(1.0, 1.0);
///
/// assert_eq!(v1.x(), 1.0);
/// assert_eq!(v1.y(), 0.0);
///
/// let two: f64 = 2.0;
/// // vectorAB = vertexB - vertexA
/// let v2_minus_v1: Vector2<f64> = v2 - v1;
///
/// assert_eq!(v2_minus_v1.norm(), 1.0);
/// assert_eq!(v2_minus_v1.unit_dir()?, Vector2::unit_y());
///
/// let mut v3 = Vertex2(0.0, 1.0);
/// // vertexA + vectorB = vertexA'
/// v3 += v2_minus_v1;
///
/// assert_eq!(v3.x(), 0.0);
/// assert_eq!(v3.y(), 2.0);
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vertex2<T: CoordsFloat>(pub T, pub T);

unsafe impl<T: CoordsFloat> Send for Vertex2<T> {}
unsafe impl<T: CoordsFloat> Sync for Vertex2<T> {}

impl<T: CoordsFloat> Vertex2<T> {
    /// Consume `self` to return inner values.
    pub fn into_inner(self) -> (T, T) {
        (self.0, self.1)
    }

    /// Return the value of the `x` coordinate of the vertex.
    pub fn x(&self) -> T {
        self.0
    }

    /// Return the value of the `y` coordinate of the vertex.
    pub fn y(&self) -> T {
        self.1
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
    /// use honeycomb_core::prelude::Vertex2;
    ///
    /// let far_far_away: Vertex2<f64> = Vertex2(2.0, 2.0);
    /// let origin: Vertex2<f64> = Vertex2::default();
    ///
    /// assert_eq!(Vertex2::average(&origin, &far_far_away), Vertex2(1.0, 1.0));
    /// ```
    pub fn average(lhs: &Vertex2<T>, rhs: &Vertex2<T>) -> Vertex2<T> {
        let two = T::from(2.0).unwrap();
        Vertex2((lhs.0 + rhs.0) / two, (lhs.1 + rhs.1) / two)
    }
}

// Building trait

impl<T: CoordsFloat> From<(T, T)> for Vertex2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self(x, y)
    }
}

// Basic operations

// -- add flavors

impl<T: CoordsFloat> std::ops::Add<Vector2<T>> for Vertex2<T> {
    // Vertex + Vector = Vertex
    type Output = Self;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<Vector2<T>> for Vertex2<T> {
    fn add_assign(&mut self, rhs: Vector2<T>) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl<T: CoordsFloat> std::ops::Add<&Vector2<T>> for Vertex2<T> {
    // Vertex + Vector = Vertex
    type Output = Self;

    fn add(self, rhs: &Vector2<T>) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<&Vector2<T>> for Vertex2<T> {
    fn add_assign(&mut self, rhs: &Vector2<T>) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

// -- sub flavors

impl<T: CoordsFloat> std::ops::Sub<Vector2<T>> for Vertex2<T> {
    // Vertex - Vector = Vertex
    type Output = Self;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<Vector2<T>> for Vertex2<T> {
    fn sub_assign(&mut self, rhs: Vector2<T>) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
    }
}

impl<T: CoordsFloat> std::ops::Sub<&Vector2<T>> for Vertex2<T> {
    // Vertex - Vector = Vertex
    type Output = Self;

    fn sub(self, rhs: &Vector2<T>) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<&Vector2<T>> for Vertex2<T> {
    fn sub_assign(&mut self, rhs: &Vector2<T>) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
    }
}

impl<T: CoordsFloat> std::ops::Sub<Vertex2<T>> for Vertex2<T> {
    type Output = Vector2<T>;

    fn sub(self, rhs: Vertex2<T>) -> Self::Output {
        Vector2(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: CoordsFloat> AttributeUpdate for Vertex2<T> {
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        Ok(Self::average(&attr1, &attr2))
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }

    // override default impl
    fn merge_incomplete(attr: Self) -> Result<Self, AttributeError> {
        Ok(attr)
    }
}

impl<T: CoordsFloat> AttributeBind for Vertex2<T> {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}
