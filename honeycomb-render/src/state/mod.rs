//! Rendering system code
//!
//! This module contains all code used to setup and continuously render available data.

// ------ MODULE DECLARATIONS

mod app;
mod camera;
mod gfx;

// ------ RE-EXPORTS

pub use app::App;

// ------ CONTENT

/// Anti-aliasing configuration enum
///
/// This enum is a bridge to the eponymous enum of the smaa [crate][SMAA]. This prevents
/// the user from adding another external crate to its project.
///
/// [SMAA]: https://github.com/fintelia/smaa-rs
#[derive(Debug, Default, Clone, Copy)]
pub enum SmaaMode {
    /// SMAA1x anti-aliasing.
    Smaa1X,
    #[default]
    /// Disabled anti-aliasing. This is the default value.
    Disabled,
}

impl From<SmaaMode> for smaa::SmaaMode {
    fn from(value: SmaaMode) -> Self {
        match value {
            SmaaMode::Smaa1X => smaa::SmaaMode::Smaa1X,
            SmaaMode::Disabled => smaa::SmaaMode::Disabled,
        }
    }
}
