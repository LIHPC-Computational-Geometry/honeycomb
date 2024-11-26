use std::collections::HashMap;

use honeycomb_core::{
    cmap::{CMap2, DartIdType, Orbit2, OrbitPolicy, VertexIdType, NULL_DART_ID},
    prelude::CoordsFloat,
};

struct Color(u8);

pub fn color<T: CoordsFloat>(cmap: &CMap2<T>) {
    // build connectivity data
    let mut vertices = cmap.fetch_vertices().identifiers.clone();
    // this can be a builtin attribute when I add a method to hijack the manager
    let mut colors: HashMap<VertexIdType, Color> = HashMap::with_capacity(vertices.len());
    let mut saturations = vec![0_u8; vertices.len()];

    while 
}

fn color_candidates() -> &[VertexIdType] {
    todo!()
}
