//! # honeycomb-render
//!
//! This crate implements a runner that can be used to display
//! combinatorial maps.
//!
//! It currently only supports 2D maps as the core library only
//! implements these (as [TwoMap])
//!
//! ## Quickstart
//!
//! The crate provides the following example:
//!
//! - `render_default_no_aa` -- Render a hardcoded arrow without anti-aliasing.
//! - `render_default_smaa1x` -- Render a hardcoded arrow with anti-aliasing.
//! - `render_splitsquaremap` -- Render a map generated using functions provided by
//!   the **honeycomb-utils** crate.
//! - `render_squaremap` -- Render a map generated using functions provided by the
//!   **honeycomb-utils** crate.

#[cfg(doc)]
use honeycomb_core::TwoMap;

// ------ MODULE DECLARATIONS

mod camera;
mod handle;
mod runner;
mod shader_data;
mod state;

// ------ RE-EXPORTS

pub use handle::RenderParameters;
pub use runner::Runner;
pub use state::SmaaMode;
