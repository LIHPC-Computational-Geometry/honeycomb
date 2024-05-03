//! mod doc

use crate::camera::{Camera, CameraController, CameraUniform, SPEED_FACTOR};
use crate::shader_data::Coords2Shader;
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{PipelineCompilationOptions, PrimitiveTopology};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct GfxState {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: PhysicalSize<u32>,
    pub(crate) render_pipeline: wgpu::RenderPipeline,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) num_vertices: u32,
    pub(crate) camera: Camera,
    pub(crate) camera_uniform: CameraUniform,
    pub(crate) camera_buffer: wgpu::Buffer,
    pub(crate) camera_bind_group: wgpu::BindGroup,
    pub(crate) camera_controller: CameraController,
    pub(crate) smaa_target: smaa::SmaaTarget,
}

impl GfxState {
    pub fn new(window: Arc<Window>, antialiasing: crate::SmaaMode) -> Self {
        let instance = wgpu::Instance::default();

        eprintln!("I: Available adapters:");
        for a in instance.enumerate_adapters(wgpu::Backends::all()) {
            eprintln!("    {:#?}", a.get_info())
        }

        // fetch window size
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let surface = instance.create_surface(window).unwrap();

        let (
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
        ) = pollster::block_on(inner(&instance, &surface, size));

        let smaa_target = smaa::SmaaTarget::new(
            &device,
            &queue,
            size.width,
            size.height,
            swapchain_format,
            smaa::SmaaMode::from(antialiasing),
        );

        let render_slice: &[Coords2Shader] = &[];

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
        }
    }
}

async fn inner(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
    size: PhysicalSize<u32>,
) -> (
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
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
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
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/shader2.wgsl"))),
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
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
            compilation_options: PipelineCompilationOptions::default(),
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
