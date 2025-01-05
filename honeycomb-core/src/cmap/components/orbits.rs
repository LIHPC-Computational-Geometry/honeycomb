/// Orbit search policy enum.
///
/// This is used to define special cases of orbits that are often used in
/// algorithms. These special cases correspond to *i-cells*.
#[derive(Debug, PartialEq, Clone)]
pub enum OrbitPolicy {
    /// 0-cell orbit.
    Vertex,
    /// 0-cell orbit, without using beta 0. This requires the cell to be complete / closed.
    VertexLinear,
    /// 1-cell orbit.
    Edge,
    /// 2-cell orbit.
    Face,
    /// 2-cell orbit, without using beta 0. This requires the cell to be complete / closed.
    FaceLinear,
    /// 3-cell orbit.
    Volume,
    /// 3-cell orbit, without using beta 0. This requires the cell to be complete / closed.
    VolumeLinear,
    /// Ordered array of beta functions that define the orbit.
    Custom(&'static [u8]),
}
