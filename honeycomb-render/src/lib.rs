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

// ------ MODULE DECLARATIONS

mod app;
mod capture;
mod gui;
mod inspector;
mod options;
mod render;
// ------ PUBLIC API

pub use app::App;

// ------ PRIVATE RE-EXPORTS

use gui::*;
use options::{resource::*, tab::*, *};
use render::{camera::*, ScenePlugin};
