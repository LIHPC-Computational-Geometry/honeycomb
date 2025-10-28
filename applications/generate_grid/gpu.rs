use cudarc::{
    driver::{CudaContext, DeviceRepr, DriverError, LaunchConfig, PushKernelArg, ValidAsZeroBits},
    nvrtc::Ptx,
};
use honeycomb::prelude::{CMap2, CMap3, CMapBuilder, CoordsFloat, Vertex2, Vertex3};
use rayon::prelude::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// -- 2D

const KERNEL_2D: &str = include_str!(concat!(env!("OUT_DIR"), "/square.ptx"));

const BLOCK_DIMS_2D: (u32, u32, u32) = (4, 4, 4);

pub fn build_2d<T: CoordsFloat>(
    [n_x, n_y]: [usize; 2],
    [len_cell_x, len_cell_y]: [f64; 2],
    split: bool,
) -> Result<CMap2<T>, DriverError> {
    if split {
        unimplemented!()
    } else {
        let grid_dims: (u32, u32, u32) = (
            (n_x as u32).div_ceil(BLOCK_DIMS_2D.0),
            (n_y as u32).div_ceil(BLOCK_DIMS_2D.1),
            1,
        );
        let n_darts = 1 + n_x * n_y * 4;

        let ctx = CudaContext::new(0)?;
        let mut betas = unsafe { ctx.alloc_pinned::<DartIdType>(3 * n_darts)? };
        let mut vertices = unsafe { ctx.alloc_pinned::<CuVertex2>(n_darts)? };

        let stream = ctx.default_stream();
        let module = ctx.load_module(Ptx::from_src(KERNEL_2D))?;
        let cfg = LaunchConfig {
            grid_dim: grid_dims,
            block_dim: BLOCK_DIMS_2D,
            shared_mem_bytes: 0,
        };
        {
            let st = stream.fork()?;
            let gen_beta = module.load_function("generate_2d_grid_betaf")?;
            let mut out_device = st.alloc_zeros::<DartIdType>(3 * n_darts)?;
            let mut launch_args = st.launch_builder(&gen_beta);
            launch_args.arg(&mut out_device);
            launch_args.arg(&n_x);
            launch_args.arg(&n_y);
            let tmp = 3 * n_darts;
            launch_args.arg(&tmp);
            unsafe { launch_args.launch(cfg.clone())? };

            st.memcpy_dtoh(&out_device, &mut betas)?;
        }
        {
            let st = stream.fork()?;
            let gen_vertices = module.load_function("generate_2d_grid_vertices")?;
            let mut out_device = st.alloc_zeros::<CuVertex2>(n_darts)?;
            let mut launch_args = st.launch_builder(&gen_vertices);
            launch_args.arg(&mut out_device);
            launch_args.arg(&len_cell_x);
            launch_args.arg(&len_cell_y);
            launch_args.arg(&n_x);
            launch_args.arg(&n_darts);
            unsafe { launch_args.launch(cfg.clone())? };

            st.memcpy_dtoh(&out_device, &mut vertices)?;
        }
        let map: CMap2<T> = CMapBuilder::<2>::from_n_darts(n_darts - 1)
            .build()
            .expect("E: unreachable");

        let betas = betas.as_slice()?;
        let bcs = betas.chunks(3).enumerate().collect::<Vec<_>>();
        bcs.into_par_iter().for_each(|(i, c)| {
            let d = i as DartIdType; // account for the null dart
            let &[b0, b1, b2] = c else { unreachable!() };
            map.set_betas(d, [b0, b1, b2]);
        });

        let vertices = vertices.as_slice()?;
        map.par_iter_vertices().for_each(|d| {
            map.force_write_vertex(d as VertexIdType, vertices[d as usize]);
        });
        Ok(map)
    }
}

// -- 3D

const KERNEL_3D: &str = include_str!(concat!(env!("OUT_DIR"), "/hex.ptx"));

const BLOCK_DIMS_3D: (u32, u32, u32) = (2, 2, 24);

pub fn build_3d<T: CoordsFloat>(
    [n_x, n_y, n_z]: [usize; 3],
    [len_cell_x, len_cell_y, len_cell_z]: [f64; 3],
    split: bool,
) -> Result<CMap3<T>, DriverError> {
    if split {
        unimplemented!()
    } else {
        let grid_dims: (u32, u32, u32) = (
            (n_x as u32).div_ceil(BLOCK_DIMS_3D.0),
            (n_y as u32).div_ceil(BLOCK_DIMS_3D.1),
            n_z as u32,
        );
        let n_darts = 1 + n_x * n_y * n_z * 24;

        let ctx = CudaContext::new(0)?;
        let mut betas = unsafe { ctx.alloc_pinned::<DartIdType>(4 * n_darts)? };
        let mut vertices = unsafe { ctx.alloc_pinned::<CuVertex3>(n_darts)? };

        let stream = ctx.default_stream();
        let module = ctx.load_module(Ptx::from_src(KERNEL_3D))?;
        let cfg = LaunchConfig {
            grid_dim: grid_dims,
            block_dim: BLOCK_DIMS_3D,
            shared_mem_bytes: 0,
        };
        {
            let st = stream.fork()?;
            let gen_beta = module.load_function("generate_hex_grid_betaf")?;
            let mut out_device = st.alloc_zeros::<DartIdType>(4 * n_darts)?;
            let mut launch_args = st.launch_builder(&gen_beta);
            launch_args.arg(&mut out_device);
            launch_args.arg(&n_x);
            launch_args.arg(&n_y);
            launch_args.arg(&n_z);
            let tmp = 4 * n_darts;
            launch_args.arg(&tmp);
            unsafe { launch_args.launch(cfg.clone())? };

            st.memcpy_dtoh(&out_device, &mut betas)?;
        }
        {
            let st = stream.fork()?;
            let gen_vertices = module.load_function("generate_hex_grid_vertices")?;
            let mut out_device = st.alloc_zeros::<CuVertex3>(n_darts)?;
            let mut launch_args = st.launch_builder(&gen_vertices);
            launch_args.arg(&mut out_device);
            launch_args.arg(&len_cell_x);
            launch_args.arg(&len_cell_y);
            launch_args.arg(&len_cell_z);
            launch_args.arg(&n_x);
            launch_args.arg(&n_y);
            launch_args.arg(&n_darts);
            unsafe { launch_args.launch(cfg.clone())? };

            st.memcpy_dtoh(&out_device, &mut vertices)?;
        }
        let map: CMap3<T> = CMapBuilder::<3>::from_n_darts(n_darts - 1)
            .build()
            .expect("E: unreachable");

        let betas = betas.as_slice()?;
        let bcs = betas.chunks_exact(4).enumerate().collect::<Vec<_>>();
        bcs.into_par_iter().for_each(|(i, c)| {
            let d = i as DartIdType; // account for the null dart
            let &[b0, b1, b2, b3] = c else { unreachable!() };
            map.set_betas(d, [b0, b1, b2, b3]);
        });

        let vertices = vertices.as_slice()?;
        map.par_iter_vertices().for_each(|d| {
            map.force_write_vertex(d as VertexIdType, vertices[d as usize]);
        });
        Ok(map)
    }
}

// -- internals

impl Default for CuVertex2 {
    fn default() -> Self {
        Self { data: [0.0; 2] }
    }
}
unsafe impl DeviceRepr for CuVertex2 {}
unsafe impl ValidAsZeroBits for CuVertex2 {}

impl<T: CoordsFloat> From<CuVertex2> for Vertex2<T> {
    fn from(value: CuVertex2) -> Self {
        let CuVertex2 { data: [x, y] } = value;
        Self(T::from(x).unwrap(), T::from(y).unwrap())
    }
}

impl Default for CuVertex3 {
    fn default() -> Self {
        Self { data: [0.0; 3] }
    }
}
unsafe impl DeviceRepr for CuVertex3 {}
unsafe impl ValidAsZeroBits for CuVertex3 {}

impl<T: CoordsFloat> From<CuVertex3> for Vertex3<T> {
    fn from(value: CuVertex3) -> Self {
        let CuVertex3 { data: [x, y, z] } = value;
        Self(
            T::from(x).unwrap(),
            T::from(y).unwrap(),
            T::from(z).unwrap(),
        )
    }
}
