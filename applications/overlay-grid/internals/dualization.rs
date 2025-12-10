use honeycomb::{
    core::{
        cmap::{CMap2, CMapBuilder},
        geometry::{CoordsFloat, Vertex2},
    },
    prelude::OrbitPolicy,
};
use rayon::prelude::*;

use crate::internals::{helpers::dart_origin, model::B2Mapping};

/// Iterates on the primal vertices to create a new dual cell around it
/// Uses hashmap for beta2 relations and hashset to avoid duplicating vertices
pub fn dualize_map<T: CoordsFloat>(map: &CMap2<T>) -> CMap2<T> {
    // Collect vertices with their adjacent vertices, filtering out boundary vertices
    let vertex_data: Vec<(u32, Vec<u32>)> = map
        .par_iter_vertices()
        .map(|vertex_id| (vertex_id, get_adjacent_vertices(map, vertex_id)))
        .filter(|(_, adj_vertices)| adj_vertices.len() >= 3)
        .collect();

    let dual_dart_count = vertex_data.len() * 4;

    let dual_map: CMap2<T> = CMapBuilder::<2>::from_n_darts(dual_dart_count)
        .build()
        .unwrap();

    let iter_vertices_zipped: Vec<(u32, Vec<u32>, u32)> = vertex_data
        .into_iter()
        .scan(0, |state, (vertex_id, adj_vertices)| {
            let current = *state;
            *state += 4;
            Some((vertex_id, adj_vertices, current + 1))
        })
        .collect();

    let par_iter_vertices = iter_vertices_zipped.into_par_iter();

    // Step 2: For each vertex in original map, create a dual face
    par_iter_vertices.for_each(|(vertex_id, adjacent_primal_vertices, new_dart)| {
        // Collect the i_cell iterators for each adjacent vertex as Vec<Vec<u32>>
        let adjacent_primal_vertices_darts: Vec<Vec<u32>> = adjacent_primal_vertices
            .iter()
            .map(|&v| map.i_cell::<0>(v).collect())
            .collect();

        // If opposite exists we beta2 link to it
        // and write centroid if doesnt exist
        for (i, adjacent_primal_vertex_darts) in adjacent_primal_vertices_darts
            .iter()
            .enumerate()
            .take(adjacent_primal_vertices.len())
        {
            let &dart = adjacent_primal_vertex_darts
                .iter()
                .find(|&&d| {
                    let opposite = map.beta::<2>(d);
                    map.vertex_id(opposite) == vertex_id
                })
                .unwrap();

            let current_new_dart = new_dart + i as u32;

            let centroid = calculate_face_centroid(map, dart);
            dual_map.force_write_vertex(current_new_dart, centroid);

            let opposite = map.beta::<2>(dart);
            map.force_write_attribute::<B2Mapping>(opposite, B2Mapping(current_new_dart));
            let attr = map.force_read_attribute::<B2Mapping>(dart);
            if let Some(attr) = attr {
                let dual_opposite_dart = attr.0;
                let _ = dual_map.force_link::<2>(current_new_dart, dual_opposite_dart);
            }
        }

        dual_map.force_link::<1>(new_dart, new_dart + 3).unwrap();
        dual_map
            .force_link::<1>(new_dart + 3, new_dart + 2)
            .unwrap();
        dual_map
            .force_link::<1>(new_dart + 2, new_dart + 1)
            .unwrap();
        dual_map.force_link::<1>(new_dart + 1, new_dart).unwrap();
    });

    dual_map
}

fn get_adjacent_vertices<T: CoordsFloat>(map: &CMap2<T>, vertex_id: u32) -> Vec<u32> {
    map.orbit(OrbitPolicy::VertexLinear, vertex_id)
        .map(|d| map.beta::<2>(d))
        .collect::<Vec<u32>>()
}

fn calculate_face_centroid<T: CoordsFloat>(map: &CMap2<T>, vertex_id: u32) -> Vertex2<T> {
    let face_darts: Vec<u32> = map.orbit(OrbitPolicy::Face, vertex_id).collect();

    let mut sum_x = T::zero();
    let mut sum_y = T::zero();
    let mut count = 0;

    for dart in face_darts {
        let vertex = dart_origin(map, dart);
        sum_x += vertex.x();
        sum_y += vertex.y();
        count += 1;
    }

    if count > 0 {
        let count_t = T::from(count).unwrap();
        Vertex2(sum_x / count_t, sum_y / count_t)
    } else {
        Vertex2(T::zero(), T::zero())
    }
}
