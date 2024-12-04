use std::{cmp::Ordering, collections::HashMap};

use honeycomb_core::{
    cmap::{CMap2, DartIdType, Orbit2, OrbitPolicy, VertexIdType},
    prelude::CoordsFloat,
    stm::atomically,
};
use itertools::Itertools;
use rayon::prelude::*;

use super::Color;

#[allow(clippy::missing_panics_doc)]
/// DSATUR algorithm implementation
///
/// This algorithm is a coloring algorithm similar to the greedy coloring algorithm.
///
/// [Reference](https://en.wikipedia.org/wiki/DSatur).
///
/// # Return
///
/// The function will return the maximal used color, i.e. the `n_color_used - 1`.
pub fn color<T: CoordsFloat>(cmap: &mut CMap2<T>) -> u8 {
    cmap.add_attribute_storage::<Color>();

    // build graph data as a collection of (Vertex, Vec<Neighbors>)
    let nodes: Vec<(VertexIdType, Vec<VertexIdType>)> = cmap
        .fetch_vertices()
        .identifiers
        .into_par_iter()
        .map(|v| {
            (
                v,
                Orbit2::new(cmap, OrbitPolicy::Vertex, v as DartIdType)
                    .flat_map(|d| {
                        [
                            cmap.vertex_id(cmap.beta::<1>(d)),
                            // needed when both nodes are on the boundary
                            cmap.vertex_id(cmap.beta::<0>(d)),
                        ]
                        .into_iter()
                    })
                    .unique()
                    .collect(),
            )
        })
        .collect();
    let mut colors: HashMap<VertexIdType, Color> = HashMap::with_capacity(nodes.len());
    let mut saturations: HashMap<VertexIdType, u8> =
        (0..nodes.len()).map(|i| (nodes[i].0, 0)).collect(); // (*)

    // find the highest degree node to start from
    let mut cmax = 0;
    let mut crt_node = nodes.iter().max_by(|n1, n2| n1.1.len().cmp(&n2.1.len()));
    let mut neigh_colors = Vec::new();
    let mut c = 0;

    while let Some((v, neighbors)) = crt_node {
        neigh_colors.extend(neighbors.iter().filter_map(|nghb| {
            *saturations.get_mut(nghb).expect("E: unreachable") += 1; // due to (*)
            colors.get(nghb)
        }));

        while neigh_colors.contains(&Color(c)) {
            c += 1;
        }
        cmax = cmax.max(c);
        colors.insert(*v, Color(c));

        // find next candidate that is:
        //
        // - not colored
        // - of highest saturation
        //   - if there are multiple, take the one of highest degree
        crt_node = nodes
            .iter()
            .filter(|(v, _)| !colors.contains_key(v))
            .max_by(|n1, n2| {
                let order = saturations[&n1.0].cmp(&saturations[&n2.0]);
                if order == Ordering::Equal {
                    n1.1.len().cmp(&n2.1.len())
                } else {
                    order
                }
            });
        neigh_colors.clear();
        c = 0;
    }

    colors.into_par_iter().for_each(|(v, c)| {
        atomically(|trans| cmap.write_attribute(trans, v, c));
    });

    cmax
}
