use std::{fs::File, io::Write};

use honeycomb_core::cmap::{CMap2, CMapBuilder};

// Init from a VTK file; only implemented for 2D map
let map: CMap2<f32> = match CMapBuilder::<2>::from_vtk_file("path/to/file.vtk").build() {
    Ok(cmap) => cmap,
    Err(e) => panic!("Error while building map: {e:?}"),
};
// Init from serialized data; implemented for 2D and 3
let map: CMap2<f32> = match CMapBuilder::<2>::from_cmap_file("path/to/file.cmap").build() {
    Ok(cmap) => cmap,
    Err(e) => panic!("Error while building map: {e:?}"),
};

// Save to VTK file; only implemented for 2D map
let file = File::create_new("out.vtk").unwrap();
map.to_vtk_binary(file);
// Serialize map data
let mut file = std::fs::File::create("out.cmap").unwrap();
let mut out = String::new();
map.serialize(&mut out);
file.write_all(out.as_bytes()).unwrap();

