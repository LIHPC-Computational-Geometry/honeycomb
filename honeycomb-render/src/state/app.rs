//! mod doc

use crate::handle::CMap2RenderHandle;
use crate::state::gfx::GfxState;
use crate::RenderParameters;
use honeycomb_core::CoordsFloat;
use std::sync::Arc;
use winit::window::Window;

pub struct App<'a, T: CoordsFloat> {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) gfx: Option<GfxState>,
    pub(crate) render_params: RenderParameters,
    pub(crate) map_handle: Option<CMap2RenderHandle<'a, T>>,
}
