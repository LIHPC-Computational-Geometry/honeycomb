//! # honeycomb-render
//!
//! This crate implements a graphical debugging tool using the `bevy` crate. The [`App`] structure
//! can be used to render a given combinatorial map, using underlying ECS logic to render map
//! items.
//!
//! All the ECS code used to render maps is left public, allowing advanced user to customize the
//! rendering tool to account for user-defined attributes.
//!
//! Note that rendering large maps may require running the program in `release` mode.
//!

// ------ CUSTOM LINTS

// more lints
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
// some exceptions
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::similar_names)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::needless_pass_by_value)] // all ECS systems are flagged with this one

// ------ MODULE DECLARATIONS

mod gui;
mod import_map;
mod options;
mod render_map;
mod scene;

// ------ PUBLIC API

// out of the box render tool

use bevy::prelude::*;
use honeycomb_core::cmap::CMap2;
use honeycomb_core::geometry::CoordsFloat;

pub fn render_2d_map<T: CoordsFloat>(cmap: CMap2<T>) {
    let mut app = App::new();
    app.insert_resource(resources::Map(cmap));
    // resource
    app.insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)));
    // plugins
    app.add_plugins(DefaultPlugins)
        .add_plugins(plugins::OptionsPlugin)
        .add_plugins(plugins::GuiPlugin)
        .add_plugins(plugins::ScenePlugin);
}

// item for custom composition

/// plugins used to build the default [`App`]
pub mod plugins {
    pub use crate::gui::GuiPlugin;
    pub use crate::options::OptionsPlugin;
    pub use crate::scene::ScenePlugin;
}

/// bundles used to build the default [`App`]
pub mod bundles {
    pub use crate::import_map::{DartBundle, EdgeBundle, FaceBundle, VertexBundle};
}

/// components used to build the default [`App`]
pub mod components {
    pub use crate::import_map::{
        Beta, Dart, DartId, Edge, EdgeId, Face, FaceId, Vertex, VertexId, Volume, VolumeId,
    };
}

/// resources used to build the default [`App`]
pub mod resources {
    pub use crate::gui::WindowVisible;
    pub use crate::import_map::{FaceNormals, Map, MapVertices, VolumeNormals};
    pub use crate::options::{
        BetaRenderColor, BetaWidth, DartHeadMul, DartRenderColor, DartShrink, DartWidth,
        EdgeRenderColor, EdgeWidth, FaceRenderColor, FaceShrink, VertexRenderColor, VertexWidth,
        VolumeRenderColor, VolumeShrink,
    };
    pub use crate::render_map::{DartGizmos, EdgeGizmos, VertexGizmos};
}

/// systems used to build the default [`App`]
pub mod systems {
    pub use crate::gui::{draw_inspected_data, draw_options, is_window_open};
    pub use crate::import_map::extract_data_from_map;
    pub use crate::render_map::{
        render_dart_enabled, render_darts, render_edge_enabled, render_edges,
        render_vertex_enabled, render_vertices,
    };
}
