//! mod doc

use crate::handle::CMap2RenderHandle;
use crate::representations::shader_data::TEST_VERTICES;
use crate::state::gfx::GfxState;
use crate::MapRef;
use crate::RenderParameters;
use honeycomb_core::CoordsFloat;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

const TARGET_FPS: f32 = 240.;

/// This yields an approximate 240 FPS
const MS_PER_FRAME: u128 = (1000. / TARGET_FPS) as u128;

pub struct App<'a, T: CoordsFloat> {
    window: Option<Arc<Window>>,
    gfx: Option<GfxState>,
    render_params: RenderParameters,
    map_handle: Option<CMap2RenderHandle<'a, T>>,
}

impl<'a, T: CoordsFloat> App<'a, T> {
    pub fn new(params: RenderParameters, map: Option<MapRef<'a, T>>) -> Self {
        Self {
            window: None,
            gfx: None,
            render_params: params,
            map_handle: map.map(|map_ref| CMap2RenderHandle::new(map_ref, Some(params))),
        }
    }
}

impl<'a, T: CoordsFloat> ApplicationHandler for App<'a, T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_attrs = Window::default_attributes().with_title("honeycomb-render");
        let window = Arc::new(event_loop.create_window(win_attrs).unwrap());

        if self.map_handle.is_some() {
            self.map_handle.as_mut().unwrap().build_intermediate();
            self.map_handle.as_mut().unwrap().build_faces();
            self.map_handle.as_mut().unwrap().build_darts();
            self.map_handle.as_mut().unwrap().build_betas();
            self.map_handle.as_mut().unwrap().save_buffered();
        }

        let gfx_state = GfxState::new(
            Arc::clone(&window),
            self.render_params.smaa_mode,
            if self.map_handle.is_some() {
                self.map_handle.as_ref().unwrap().vertices()
            } else {
                TEST_VERTICES
            },
        );
        window.request_redraw();

        self.window = Some(window);
        self.gfx = Some(gfx_state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().unwrap().id() == window_id
            && !self.gfx.as_mut().unwrap().input(&event, event_loop)
        {
            match event {
                WindowEvent::Resized(new_size) => {
                    self.gfx.as_mut().unwrap().resize(Some(new_size));
                    self.window.as_ref().unwrap().request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    let start = std::time::Instant::now();
                    self.gfx.as_mut().unwrap().update();
                    match self.gfx.as_mut().unwrap().render(None) {
                        Ok(()) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            self.gfx.as_mut().unwrap().resize(None);
                            self.window.as_ref().unwrap().request_redraw();
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(), // kill if OOM
                        Err(e) => eprintln!("{e:?}"),
                    };
                    // put a hard cap on the rendering speed
                    std::thread::sleep(std::time::Duration::from_millis(
                        MS_PER_FRAME.saturating_sub(start.elapsed().as_millis()) as u64,
                    ));
                }
                WindowEvent::CloseRequested => event_loop.exit(),
                _ => {}
            };
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
    }
}
