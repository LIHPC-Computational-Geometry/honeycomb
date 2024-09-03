//!

pub use honeycomb_core as core;

#[cfg(feature = "kernels")]
pub use honeycomb_kernels as kernels;

#[cfg(feature = "render")]
pub use honeycomb_render as render;

pub mod prelude {
    pub use honeycomb_core::prelude::*;

    #[cfg(feature = "kernels")]
    pub use honeycomb_kernels::grisubal;

    #[cfg(feature = "render")]
    pub use honeycomb_render::App;
}
