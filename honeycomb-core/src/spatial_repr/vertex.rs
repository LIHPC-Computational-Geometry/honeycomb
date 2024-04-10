//! Custom spatial representation
//!
//! This module contains all code used to model vertices as wrappers of a common
//! type ([Coords2]).
//!

// ------ IMPORTS

use crate::{
    AttributeBind, AttributeUpdate, Coords2, CoordsFloat, OrbitPolicy, Vector2, VertexIdentifier,
};

// ------ CONTENT

/// 2D vertex representation
///
/// This structure is a wrapper around a [Coords2] value. Defining this as a wrapper
/// instead of a simple type alias allow us to introduce the notion of homogeneity.
///
/// # Generics
///
/// - `T: CoordsFloat` -- Generic type for coordinates representation.
///
/// # Example
///
/// ```
/// # use honeycomb_core::CoordsError;
/// # fn main() -> Result<(), CoordsError> {
/// use honeycomb_core::{Vector2, Vertex2, FloatType};
///
/// let v1 = Vertex2::from((1.0, 0.0));
/// let v2 = Vertex2::from((1.0, 1.0));
///
/// assert_eq!(v1.x(), 1.0);
/// assert_eq!(v1.y(), 0.0);
///
/// let two: FloatType = 2.0;
/// // vectorAB = vertexB - vertexA
/// let v2_minus_v1: Vector2<FloatType> = v2 - v1;
///
/// assert_eq!(v2_minus_v1.norm(), 1.0);
/// assert_eq!(v2_minus_v1.unit_dir()?, Vector2::unit_y());
///
/// let mut v3 = Vertex2::from((0.0, 1.0));
/// // vertexA + vectorB = vertexA'
/// v3 += v2_minus_v1;
///
/// assert_eq!(v3.x(), 0.0);
/// assert_eq!(v3.y(), 2.0);
///
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vertex2<T: CoordsFloat> {
    inner: Coords2<T>,
}

impl<T: CoordsFloat> Vertex2<T> {
    /// Consume `self` to return inner value
    ///
    /// # Return
    ///
    /// Return a [Coords2] object.
    ///
    pub fn into_inner(self) -> Coords2<T> {
        self.inner
    }

    /// Getter
    ///
    /// # Return
    ///
    /// Return the value of the `x` coordinate of the vertex.
    ///
    pub fn x(&self) -> T {
        self.inner.x
    }

    /// Getter
    ///
    /// # Return
    ///
    /// Return the value of the `y` coordinate of the vertex.
    ///
    pub fn y(&self) -> T {
        self.inner.y
    }

    /// Compute the mid-point between two vertices.
    ///
    /// # Return
    ///
    /// Return the mid-point as a new [Vertex2] object.
    ///
    /// # Example
    ///
    /// ```rust
    /// use honeycomb_core::Vertex2;
    ///
    /// let far_far_away: Vertex2<f64> = Vertex2::from((2.0, 2.0));
    /// let origin: Vertex2<f64> = Vertex2::default();
    ///
    /// assert_eq!(Vertex2::average(&origin, &far_far_away), Vertex2::from((1.0, 1.0)));
    /// ```
    pub fn average(lhs: &Vertex2<T>, rhs: &Vertex2<T>) -> Vertex2<T> {
        let two = T::from(2.0).unwrap();
        Vertex2::from(((lhs.x() + rhs.x()) / two, (lhs.y() + rhs.y()) / two))
    }
}

// Building traits

macro_rules! impl_from_for_vertex {
    ($src_type: ty) => {
        impl<T: CoordsFloat> From<$src_type> for Vertex2<T> {
            fn from(value: $src_type) -> Self {
                Self {
                    inner: Coords2::from(value),
                }
            }
        }
    };
}

impl_from_for_vertex!((T, T));
impl_from_for_vertex!([T; 2]);
impl_from_for_vertex!(Coords2<T>);

// Basic operations

// -- add flavors

impl<T: CoordsFloat> std::ops::Add<Vector2<T>> for Vertex2<T> {
    // Vertex + Vector = Vertex
    type Output = Self;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        Self {
            inner: self.inner + rhs.into_inner(),
        }
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<Vector2<T>> for Vertex2<T> {
    fn add_assign(&mut self, rhs: Vector2<T>) {
        self.inner += rhs.into_inner()
    }
}

impl<T: CoordsFloat> std::ops::Add<&Vector2<T>> for Vertex2<T> {
    // Vertex + Vector = Vertex
    type Output = Self;

    fn add(self, rhs: &Vector2<T>) -> Self::Output {
        let mut tmp = self.inner;
        tmp.x += rhs.x();
        tmp.y += rhs.y();
        Self { inner: tmp }
    }
}

impl<T: CoordsFloat> std::ops::AddAssign<&Vector2<T>> for Vertex2<T> {
    fn add_assign(&mut self, rhs: &Vector2<T>) {
        self.inner.x += rhs.x();
        self.inner.y += rhs.y();
    }
}

// -- sub flavors

impl<T: CoordsFloat> std::ops::Sub<Vector2<T>> for Vertex2<T> {
    // Vertex - Vector = Vertex
    type Output = Self;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        Self {
            inner: self.inner - rhs.into_inner(),
        }
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<Vector2<T>> for Vertex2<T> {
    fn sub_assign(&mut self, rhs: Vector2<T>) {
        self.inner -= rhs.into_inner();
    }
}

impl<T: CoordsFloat> std::ops::Sub<&Vector2<T>> for Vertex2<T> {
    // Vertex - Vector = Vertex
    type Output = Self;

    fn sub(self, rhs: &Vector2<T>) -> Self::Output {
        let mut tmp = self.inner;
        tmp.x -= rhs.x();
        tmp.y -= rhs.y();
        Self { inner: tmp }
    }
}

impl<T: CoordsFloat> std::ops::SubAssign<&Vector2<T>> for Vertex2<T> {
    fn sub_assign(&mut self, rhs: &Vector2<T>) {
        self.inner.x -= rhs.x();
        self.inner.y -= rhs.y();
    }
}

impl<T: CoordsFloat> std::ops::Sub<Vertex2<T>> for Vertex2<T> {
    type Output = Vector2<T>;

    fn sub(self, rhs: Vertex2<T>) -> Self::Output {
        Vector2::from(self.into_inner() - rhs.into_inner())
    }
}

/// Attribute logic definitions
///
/// - **MERGING POLICY** - The new vertex is placed at the midpoint between the two existing ones.
/// - **SPLITTING POLICY** - The current vertex is duplicated.
/// - **UNDEFINED ATTRIBUTES MERGING** - The new vertex takes the value of the one provided if it
/// exists, otherwise the function panics.
impl<T: CoordsFloat> AttributeUpdate for Vertex2<T> {
    fn merge(attr1: Self, attr2: Self) -> Self {
        Self::average(&attr1, &attr2)
    }

    fn split(attr: Self) -> (Self, Self) {
        (attr, attr)
    }

    fn merge_undefined(attr: Option<Self>) -> Self {
        attr.unwrap()
    }
}

/// Attribute support definitions
///
/// - **BINDS TO 0-CELLS**
impl<T: CoordsFloat> AttributeBind for Vertex2<T> {
    type IdentifierType = VertexIdentifier;

    fn binds_to<'a>() -> OrbitPolicy<'a> {
        OrbitPolicy::Vertex
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FloatType;

    #[test]
    fn add_vertex_vector() {
        let mut a: Vertex2<FloatType> = Vertex2::from((1.0, 1.0));
        let b: Vector2<FloatType> = Vector2::from((1.0, 0.0));
        let a_moved = a + b;
        assert_eq!(a_moved, Vertex2::from((2.0, 1.0)));
        a += &b;
        assert_eq!(a, a_moved);
        a += b;
        assert_eq!(a, Vertex2::from((3.0, 1.0)));
    }

    #[test]
    fn sub_vertex_vector() {
        let mut a: Vertex2<FloatType> = Vertex2::from((1.0, 1.0));
        let b: Vector2<FloatType> = Vector2::from((1.0, 0.0));
        let a_moved = a - b;
        assert_eq!(a_moved, Vertex2::from((0.0, 1.0)));
        a -= &b;
        assert_eq!(a, a_moved);
        a -= b;
        assert_eq!(a, Vertex2::from((-1.0, 1.0)));
    }

    #[test]
    fn sub_vertex_vertex() {
        let a: Vertex2<FloatType> = Vertex2::from((1.0, 1.0));
        let b: Vertex2<FloatType> = Vertex2::from((1.0, 0.0));
        let ab = b - a;
        assert_eq!(ab, Vector2::from((0.0, -1.0)))
    }
}
