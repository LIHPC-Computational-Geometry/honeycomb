use honeycomb_core::prelude::{CoordsFloat, Vertex2};

/// Compute the quality factor of a given triangle.
pub fn quality_tri<T: CoordsFloat>(v1: &Vertex2<T>, v2: &Vertex2<T>, v3: &Vertex2<T>) -> T {
    let mut signed_area =
        v1.x() * (v2.y() - v3.y()) + v2.x() * (v3.y() - v1.y()) + v3.x() * (v1.y() - v2.y());
    signed_area /= T::from(2.0).unwrap();

    let squared_edge_sum = (v2.x() - v1.x()).powi(2)
        + (v2.y() - v1.y()).powi(2)
        + (v3.x() - v2.x()).powi(2)
        + (v3.y() - v2.y()).powi(2)
        + (v1.x() - v3.x()).powi(2)
        + (v1.y() - v3.y()).powi(2);
    T::from(4.0 * 3.0_f64.sqrt()).unwrap() * signed_area / squared_edge_sum
}

/// Compute the quality factor of a given simple polygon.
pub fn quality_any<T: CoordsFloat>(vertices: &[Vertex2<T>]) -> T {
    let last = &vertices[vertices.len() - 1];
    let blast = &vertices[vertices.len() - 2];
    let first = &vertices[0];
    // shoelace formula
    let mut signed_area = vertices
        .windows(3)
        .map(|sl| {
            let [v1, v2, v3] = sl else { unreachable!() };
            v2.x() * (v3.y() - v1.y())
        })
        .fold(T::zero(), |t1, t2| t1 + t2);
    signed_area += last.x() * (first.y() - blast.y());
    signed_area += first.x() * (vertices[1].y() - last.y());
    signed_area /= T::from(2.0).unwrap();

    let mut squared_edge_sum = vertices
        .windows(2)
        .map(|sl| {
            let [v1, v2] = sl else { unreachable!() };
            (v2.x() - v1.x()).powi(2) + (v2.y() - v1.y()).powi(2)
        })
        .fold(T::zero(), |t1, t2| t1 + t2);
    squared_edge_sum += (first.x() - last.x()).powi(2) + (first.y() - last.y()).powi(2);
    T::from(4.0 * 3.0_f64.sqrt()).unwrap() * signed_area / squared_edge_sum
}
