//! TODO

use honeycomb_core::{
    cmap::{CMap2, DartIdType},
    geometry::CoordsFloat,
    stm::atomically,
};

/// TODO
pub fn compute_faces_skewness<'a, 'b, T: CoordsFloat>(
    map: &'a CMap2<T>,
) -> impl Iterator<Item = T> + 'b
where
    'a: 'b,
{
    map.iter_faces().map(|fid| {
        let (mut d1, mut d2, mut d3) = (
            fid as DartIdType,
            map.beta::<1>(fid as DartIdType),
            map.beta::<1>(map.beta::<1>(fid as DartIdType)),
        );
        let (mut vid1, mut vid2, mut vid3) =
            (map.vertex_id(d1), map.vertex_id(d2), map.vertex_id(d3));
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
        let ideal_theta = T::from((cnt - 2) as f64 * std::f64::consts::PI / cnt as f64).unwrap();

        ((max_theta - ideal_theta) / (T::from(std::f64::consts::PI).unwrap() - ideal_theta))
            .max((ideal_theta - min_theta) / ideal_theta)
    })
}

#[cfg(test)]
mod tests;
