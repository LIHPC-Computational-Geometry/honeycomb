use std::{cmp::Ordering, collections::HashMap};

use honeycomb_core::{
    cmap::{CMap2, DartIdType, Orbit2, OrbitPolicy, VertexIdType, NULL_DART_ID},
    prelude::CoordsFloat,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Color(u8);

/// DSATUR algorithm implementation
pub fn color<T: CoordsFloat>(cmap: &CMap2<T>) {
    // build graph data as a collection of (Vertex, Vec<Neighbors>)
    let nodes: Vec<(VertexIdType, Vec<VertexIdType>)> = cmap
        .fetch_vertices()
        .identifiers
        .into_iter()
        .filter_map(|v| {
            if Orbit2::new(cmap, OrbitPolicy::Vertex, v as DartIdType)
                .any(|d| cmap.beta::<2>(d) == NULL_DART_ID)
            {
                None
            } else {
                Some((
                    v,
                    Orbit2::new(cmap, OrbitPolicy::Vertex, v as DartIdType)
                        .map(|d| cmap.vertex_id(cmap.beta::<2>(d)))
                        .collect(),
                ))
            }
        })
        .collect();
    // this can be a builtin attribute when I add a method to hijack the manager
    let mut colors: HashMap<VertexIdType, Color> = HashMap::with_capacity(nodes.len());
    let mut saturations: HashMap<VertexIdType, u8> =
        (0..nodes.len()).map(|i| (nodes[i].0, 0)).collect();

    // find the highest degree node to start from
    let mut crt_node = nodes.iter().max_by(|n1, n2| n1.1.len().cmp(&n2.1.len()));

    while let Some((v, neighbors)) = crt_node {
        let neigh_colors: Vec<Color> = neighbors
            .iter()
            .filter_map(|nghb| {
                *saturations.get_mut(nghb).unwrap() += 1;
                colors.get(nghb)
            })
            .copied()
            .collect();

        let mut tmp = 0;
        while neigh_colors.contains(&Color(tmp)) {
            tmp += 1;
        }

        colors.insert(*v, Color(tmp));

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
    }
}
