use honeycomb::prelude::*;

use honeycomb::kernels::utils::is_orbit_orientation_consistent;
use honeycomb::stm::{abort, atomically_with_err};

pub fn smooth_vertex_checked(map: &CMap2<f64>, vid: VertexIdType) -> Result<(), &'static str> {
    // neighbors of the vertex: follow beta_2 around the vertex orbit
    let neighbors: Vec<VertexIdType> = map
        .orbit(OrbitPolicy::Vertex, vid as DartIdType)
        .map(|d| map.vertex_id(map.beta::<2>(d)))
        .collect();

    // group the operations in a single transaction: they apply all-or-nothing
    atomically_with_err(|t: &mut Transaction| {
        // average of the neighbors' positions
        let mut new_pos = Vertex2::default();
        for n in &neighbors {
            // reads and writes inside a transaction take a reference to the transaction
            // as argument for bookkeeping; note the `tx` method suffix and internal
            // transaction error handling with `?`
            let v = if let Some(val) = map.read_vertex_tx(t, *n)? {
                val
            } else {
                abort("E: no coordinates")?;
            };
            new_pos.0 += v.0;
            new_pos.1 += v.1;
        }
        new_pos.0 /= neighbors.len() as f64;
        new_pos.1 /= neighbors.len() as f64;
        map.write_vertex_tx(t, vid, new_pos)?;

        // cancel the whole transaction (rollback) if a face was reversed
        if !is_orbit_orientation_consistent(t, map, vid)? {
            abort("E: smoothing would reverse a face")?;
        }
        Ok(())
    })
}
