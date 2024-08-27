//! # honeycomb-render
//!
//! WIP
//!
//! ## Quickstart
//!
//! Examples are available in the dedicated crate.

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

mod app;
mod capture;
mod gui;
mod inspector;
mod options;
mod render;
// ------ PUBLIC API

// out of the box render tool

pub use app::App;

// item for custom composition

/// plugins used to build the default [`App`]
pub mod plugins {
    pub use crate::capture::CapturePlugin;
    pub use crate::gui::GuiPlugin;
    pub use crate::options::OptionsPlugin;
    pub use crate::render::ScenePlugin;
}

/// bundles used to build the default [`App`]
pub mod bundles {
    pub use crate::capture::ecs_data::{
        DartBodyBundle, DartHeadBundle, EdgeBundle, FaceBundle, VertexBundle,
    };
}

/// components used to build the default [`App`]
pub mod components {
    pub use crate::capture::ecs_data::{
        Beta, CaptureId, DartBody, DartHead, DartId, Edge, EdgeId, Face, FaceId, Vertex, VertexId,
        Volume, VolumeId,
    };
    pub use crate::render::camera::PanOrbitCamera;
}

/// resources used to build the default [`App`]
pub mod resources {
    pub use crate::capture::ecs_data::{FaceNormals, MapVertices};
    pub use crate::options::resource::*;
}

/// systems used to build the default [`App`]
pub mod systems {
    pub use crate::capture::system::*;
    pub use crate::inspector::tab::draw_inspected_data;
    pub use crate::options::tab::draw_options;
    pub use crate::render::{
        camera::{cursor_in_render, update_camera},
        picking::update_picking,
        scene::setup_scene,
        update::*,
    };
}
