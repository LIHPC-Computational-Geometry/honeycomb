/// # Orbit search policy enum
///
/// This is used to define special cases of orbits that are often used in
/// algorithms. These special cases correspond to `i`-cells.
#[derive(Debug, PartialEq, Clone)]
pub enum OrbitPolicy {
    /// 0-cell orbit.
    Vertex,
    /// 0-cell orbit, without using beta 0. Incorrect if the cell isn't complete / closed.
    VertexLinear,
    /// 1-cell orbit.
    Edge,
    /// 2-cell orbit.
    Face,
    /// 2-cell orbit, without using beta 0. Incorrect if the cell isn't complete / closed.
    FaceLinear,
    /// 3-cell orbit.
    Volume,
    /// 3-cell orbit, without using beta 0. Incorrect if the cell isn't complete / closed.
    VolumeLinear,
    /// Ordered array of beta functions defining the orbit.
    Custom(&'static [u8]),
}

/// Custom fallible `unfold` iterator.
///
/// This doesn't solve usability issues, but it reduces internal boilerplate by
/// allowing the usage of `?` inside the closure.
///
/// Modelled after [`std::iter::FromFn`].
#[derive(Clone)]
pub(crate) struct TryFromFn<F>(F);

impl<T, E, F> Iterator for TryFromFn<F>
where
    F: FnMut() -> Result<Option<T>, E>,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.0)() {
            Ok(Some(value)) => Some(Ok(value)), // Yield a successful item
            Ok(None) => None,                   // End iteration
            Err(e) => Some(Err(e)),             // Yield an error
        }
    }
}

pub(crate) fn try_from_fn<T, E, F>(f: F) -> TryFromFn<F>
where
    F: FnMut() -> Result<Option<T>, E>,
{
    TryFromFn(f)
}
