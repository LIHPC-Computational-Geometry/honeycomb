use std::io::Write;
#[cfg(feature = "thread-binding")]
use std::sync::Arc;

use clap::Parser;
use honeycomb::prelude::{CMap2, CoordsFloat};
#[cfg(feature = "thread-binding")]
use hwlocality::{Topology, cpu::binding::CpuBindingFlags};
use rayon::ThreadPoolBuilder;

#[cfg(feature = "thread-binding")]
use honeycomb_benches::utils::get_physical_cores;
use honeycomb_benches::{
    cli::{Benches, Cli, Format},
    cut_edges::bench_cut_edges,
    grid_gen::bench_generate_2d_grid,
    grisubal::bench_grisubal,
    remesh::bench_remesh,
    shift::bench_shift,
};

fn main() {
    let cli = Cli::parse();

    let n_t = if let Some(val) = cli.n_threads {
        val.get()
    } else {
        std::thread::available_parallelism()
            .map(|v| v.get())
            .unwrap_or(1)
    };
    let builder = ThreadPoolBuilder::new().num_threads(n_t);

    #[cfg(feature = "thread-binding")]
    {
        if cli.bind_threads {
            // build the topology & check that all necessary features are available on the machine
            let topology = Topology::new().unwrap();
            let topology = Arc::new(topology);
            match get_physical_cores(&topology) {
                Ok(mut cores) => {
                    let n_t = if cores.len() < n_t {
                        // don't allow more than one thread per physical core
                        // this is sane since this branch executes only if we explicitly enable binding
                        eprintln!(
                            "W: Less physical cores than logical threads; proceeding with one thread per core ({})",
                            cores.len()
                        );
                        cores.len()
                    } else {
                        n_t
                    };
                    builder
                        .num_threads(n_t)
                        .spawn_handler(|t_builder| {
                            // master thread
                            let topology = topology.clone();
                            // safe to unwrap due to n_t value adjustment
                            let core = cores.pop_front().expect("E: unreachable");
                            let mut bind_to = core.cpuset().unwrap().clone_target();
                            bind_to.singlify();
                            std::thread::spawn(move || {
                                // worker thread
                                let tid = hwlocality::current_thread_id();
                                topology
                                    .bind_thread_cpu(tid, &bind_to, CpuBindingFlags::empty())
                                    .unwrap();

                                // do the work
                                t_builder.run()
                            });
                            Ok(())
                        })
                        .build_global()
                        .unwrap();
                }
                Err(e) => {
                    eprintln!("W: {e}");
                    builder.build_global().unwrap();
                }
            }
        } else {
            builder.build_global().unwrap();
        }
    }
    #[cfg(not(feature = "thread-binding"))]
    {
        builder.build_global().unwrap();
    }

    if cli.simple_precision {
        run_benchmarks::<f32>(cli, n_t);
    } else {
        run_benchmarks::<f64>(cli, n_t);
    }
}

fn run_benchmarks<T: CoordsFloat>(cli: Cli, n_threads: usize) {
    #[allow(unused_mut, unused_assignments)]
    let mut bind_threads = false;

    #[cfg(feature = "thread-binding")]
    {
        bind_threads = cli.bind_threads;
    }

    let map: CMap2<T> = match cli.benches {
        Benches::Generate2dGrid(args) => bench_generate_2d_grid(args),
        Benches::CutEdges(args) => bench_cut_edges(args, n_threads, bind_threads),
        Benches::Grisubal(args) => bench_grisubal(args),
        Benches::Remesh(args) => bench_remesh(args),
        Benches::Shift(args) => bench_shift(args),
    };
    // all bench currently generate a map,
    // we may have to move this to match arms if this changes
    if let Some(f) = cli.save_as {
        match f {
            Format::Cmap => {
                // FIXME: update serialize sig
                let mut out = String::new();
                let mut file = std::fs::File::create("out.cmap").unwrap();
                map.serialize(&mut out);
                file.write_all(out.as_bytes()).unwrap();
            }
            Format::Vtk => {
                let mut file = std::fs::File::create("out.vtk").unwrap();
                map.to_vtk_binary(&mut file);
            }
        }
    } else {
        std::hint::black_box(map);
    }
}
