//! Rendering system code
//!
//! This module contains all code used to setup and continuously render available data.

// ------ IMPORTS

// intern
use crate::camera::{Camera, CameraController, CameraUniform, SPEED_FACTOR};
use crate::handle::CMap2RenderHandle;
use crate::shader_data::{Coords2Shader, TEST_VERTICES};
use crate::RenderParameters;
use honeycomb_core::{CMap2, CoordsFloat};

// extern
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use wgpu::PrimitiveTopology;
use winit::dpi::PhysicalSize;
use winit::{event::WindowEvent, window::Window};

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

pub struct State<'a, T: CoordsFloat> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    smaa_target: smaa::SmaaTarget,
    map_handle: Option<CMap2RenderHandle<'a, T>>,
    window: &'a Window,
}

async fn inner(
    window: &Window,
    size: PhysicalSize<u32>,
) -> (
    wgpu::Surface<'_>,
    wgpu::Device,
    wgpu::Queue,
    wgpu::SurfaceConfiguration,
    Camera,
    CameraUniform,
    wgpu::Buffer,
    wgpu::BindGroup,
    CameraController,
    wgpu::TextureFormat,
    wgpu::RenderPipeline,
) {
    let instance = wgpu::Instance::default();

    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("I: Available adapters:");
        for a in instance.enumerate_adapters(wgpu::Backends::all()) {
            eprintln!("    {:#?}", a.get_info())
        }
    }

    let surface = instance.create_surface(window).unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("E: Failed to fetch appropriate adaptater");

    eprintln!("I: Selected adapter: {:#?}", adapter.get_info());

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("E: Failed to create device");

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/shader2.wgsl"))),
    });

    let config = surface
        .get_default_config(&adapter, size.width, size.height)
        .unwrap();
    surface.configure(&device, &config);

    // Camera work

    let camera = Camera {
        // position the camera 1 unit up and 2 units back
        // +z is out of the screen
        eye: (0.0, 0.0, 3.0).into(),
        // have it look at the origin
        target: (0.0, 0.0, 0.0).into(),
        // which way is "up"
        up: cgmath::Vector3::unit_y(),
        aspect: config.width as f32 / config.height as f32,
        fovy: 45.0,
        znear: 0.1,
        zfar: 100.0,
    };

    let mut camera_uniform = CameraUniform::default();
    camera_uniform.update_view_proj(&camera);

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
        label: Some("camera_bind_group"),
    });

    let camera_controller = CameraController::new(SPEED_FACTOR * 3.0);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&camera_bind_group_layout],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Coords2Shader::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: Default::default(),
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: Default::default(),
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    (
        surface,
        device,
        queue,
        config,
        camera,
        camera_uniform,
        camera_buffer,
        camera_bind_group,
        camera_controller,
        swapchain_format,
        render_pipeline,
    )
}

impl<'a, T: CoordsFloat> State<'a, T> {
    pub async fn new(
        window: &'a Window,
        render_params: RenderParameters,
        map: &'a CMap2<T>,
    ) -> Self {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let (
            surface,
            device,
            queue,
            config,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            swapchain_format,
            render_pipeline,
        ) = inner(window, size).await;

        let smaa_target = smaa::SmaaTarget::new(
            &device,
            &queue,
            window.inner_size().width,
            window.inner_size().height,
            swapchain_format,
            smaa::SmaaMode::from(render_params.smaa_mode),
        );

        let mut map_handle = CMap2RenderHandle::new(map, Some(render_params));
        map_handle.build_intermediate();
        map_handle.build_darts();
        map_handle.build_faces();
        // map_handle.build_betas();
        map_handle.save_buffered();

        let render_slice = map_handle.vertices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(render_slice),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_vertices = render_slice.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            vertex_buffer,
            render_pipeline,
            num_vertices,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            smaa_target,
            map_handle: Some(map_handle),
            window,
        }
    }

    pub async fn new_test(window: &'a Window, render_params: RenderParameters) -> Self {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let (
            surface,
            device,
            queue,
            config,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            swapchain_format,
            render_pipeline,
        ) = inner(window, size).await;

        let smaa_target = smaa::SmaaTarget::new(
            &device,
            &queue,
            window.inner_size().width,
            window.inner_size().height,
            swapchain_format,
            smaa::SmaaMode::from(render_params.smaa_mode),
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(TEST_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_vertices = TEST_VERTICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            vertex_buffer,
            render_pipeline,
            num_vertices,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            smaa_target,
            map_handle: None,
            window,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(&mut self, new_size_opt: Option<PhysicalSize<u32>>) {
        let new_size = new_size_opt.unwrap_or(self.size);
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.window.request_redraw();
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // update the current number of vertices
        if let Some(handle) = &self.map_handle {
            self.num_vertices = handle.vertices().len() as u32;
        };
        // render
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let smaa_frame = self
            .smaa_target
            .start_frame(&self.device, &self.queue, &view);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &smaa_frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        smaa_frame.resolve();
        frame.present();
        Ok(())
    }
}
