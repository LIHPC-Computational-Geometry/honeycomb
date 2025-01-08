/// # Orbit search policy enum
///
/// This is used to define special cases of orbits that are often used in
/// algorithms. These special cases correspond to `i`-cells.
#[derive(Debug, PartialEq)]
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
    /// Ordered array of beta functions defining the orbit.
    Custom(&'static [u8]),
}
