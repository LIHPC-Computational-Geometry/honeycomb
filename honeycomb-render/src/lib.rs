//! # honeycomb-render
//!
//! This crate implements a runner that can be used to display combinatorial maps.
//!
//! It currently only supports 2D maps as the core library only implements these (as [`honeycomb_core::CMap2`])
//!
//! ## Key bindings
//!
//! - Directional arrows -- Move up, down, left and right
//! - `F` -- Move forward (i.e. zoom in)
//! - `B` -- Move backward (i.e. zoom out)
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
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]

// ------ MODULE DECLARATIONS

mod handle;
mod runner;
mod state;

mod representations;

// ------ RE-EXPORTS

pub use handle::RenderParameters;
pub use runner::{launch, launch_async};
pub use state::SmaaMode;

// ------ CONTENT

/// Convenience type alias
type MapRef<'a, T> = &'a honeycomb_core::CMap2<T>;
