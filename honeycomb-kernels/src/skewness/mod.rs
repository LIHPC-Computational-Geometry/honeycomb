//! Skewness computation routines.
//!
//! We use the equiangular skew formula presented
//! [here](https://en.wikipedia.org/wiki/Types_of_mesh#Skewness)

use honeycomb_core::{
    cmap::{CMap2, CMap3, DartIdType, FaceIdType},
    geometry::CoordsFloat,
    stm::atomically,
};

/// Return the skewness of a given face.
///
/// # Arguments
///
/// - `map: &CMap2<T>` -- Input map.
/// - `fid: FaceIdType` -- Face to compute the skewness of.
///
/// # Return
///
/// The value returned is comprised in the `[0.0; 1.0]` range. `0.0` corresponds to ideal
/// (equilateral) cells while `1.0` corresponds to degenerate cells.
///
/// # Panics
///
/// This function will panic if a topological vertex has no associated coordinates.
// #[inline] // bench and adjust
#[must_use = "unused return value"]
pub fn compute_face_skewness_2d<T: CoordsFloat>(map: &CMap2<T>, fid: FaceIdType) -> T {
    let (mut d1, mut d2, mut d3) = (
        fid as DartIdType,
        map.beta::<1>(fid as DartIdType),
        map.beta::<1>(map.beta::<1>(fid as DartIdType)),
    );
    let (mut vid1, mut vid2, mut vid3) = (map.vertex_id(d1), map.vertex_id(d2), map.vertex_id(d3));
    let mut cnt = 0;
    let mut min_theta = T::max_value();
    let mut max_theta = T::min_value();
    loop {
        let theta = atomically(|t| {
            let v1 = map.read_vertex(t, vid1)?.unwrap();
            let v2 = map.read_vertex(t, vid2)?.unwrap();
            let v3 = map.read_vertex(t, vid3)?.unwrap();
            let vin = v1 - v2;
            let vout = v3 - v2;
            Ok(T::acos(vin.dot(&vout) / (vin.norm() * vout.norm())))
        });
        min_theta = min_theta.min(theta);
        max_theta = max_theta.max(theta);
        // move forward
        cnt += 1;
        d1 = d2;
        d2 = d3;
        d3 = map.beta::<1>(d3);
        vid1 = vid2;
        vid2 = vid3;
        vid3 = map.vertex_id(d3);
        if d1 == fid as DartIdType {
            break;
        }
    }
    let ideal_theta = T::from(f64::from(cnt - 2) * std::f64::consts::PI / f64::from(cnt)).unwrap();

    ((max_theta - ideal_theta) / (T::from(std::f64::consts::PI).unwrap() - ideal_theta))
        .max((ideal_theta - min_theta) / ideal_theta)
}

/// Return the skewness of a given face.
///
/// # Arguments
///
/// - `map: &CMap3<T>` -- Input map.
/// - `fid: FaceIdType` -- Face to compute the skewness of.
///
/// # Return
///
/// The value returned is comprised in the `[0.0; 1.0]` range. `0.0` corresponds to ideal
/// (equilateral) cells while `1.0` corresponds to degenerate cells.
///
/// # Panics
///
/// This function will panic if a topological vertex has no associated coordinates.
// #[inline] // bench and adjust
#[must_use = "unused return value"]
pub fn compute_face_skewness_3d<T: CoordsFloat>(map: &CMap3<T>, fid: FaceIdType) -> T {
    let (mut d1, mut d2, mut d3) = (
        fid as DartIdType,
        map.beta::<1>(fid as DartIdType),
        map.beta::<1>(map.beta::<1>(fid as DartIdType)),
    );
    let (mut vid1, mut vid2, mut vid3) = (map.vertex_id(d1), map.vertex_id(d2), map.vertex_id(d3));
    let mut cnt = 0;
    let mut min_theta = T::max_value();
    let mut max_theta = T::min_value();

    loop {
        let theta = atomically(|t| {
            let v1 = map.read_vertex(t, vid1)?.unwrap();
            let v2 = map.read_vertex(t, vid2)?.unwrap();
            let v3 = map.read_vertex(t, vid3)?.unwrap();
            let vin = v1 - v2;
            let vout = v3 - v2;
            Ok(T::acos(vin.dot(&vout) / (vin.norm() * vout.norm())))
        });
        min_theta = min_theta.min(theta);
        max_theta = max_theta.max(theta);
        // move forward
        cnt += 1;
        d1 = d2;
        d2 = d3;
        d3 = map.beta::<1>(d3);
        vid1 = vid2;
        vid2 = vid3;
        vid3 = map.vertex_id(d3);
        if d1 == fid as DartIdType {
            break;
        }
    }
    let ideal_theta = T::from(f64::from(cnt - 2) * std::f64::consts::PI / f64::from(cnt)).unwrap();

    ((max_theta - ideal_theta) / (T::from(std::f64::consts::PI).unwrap() - ideal_theta))
        .max((ideal_theta - min_theta) / ideal_theta)
}

#[cfg(test)]
mod tests;
