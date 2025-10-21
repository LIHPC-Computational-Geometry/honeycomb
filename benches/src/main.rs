use std::io::Write;

use clap::Parser;
use honeycomb::prelude::{CMap2, CoordsFloat};

#[cfg(all(not(target_env = "msvc"), feature = "jemalloc"))]
use tikv_jemallocator::Jemalloc;

use honeycomb_benches::{
    cli::{Benches, Cli, Format},
    grisubal::bench_grisubal,
    prof_init, prof_start, prof_stop,
    remesh::bench_remesh,
    shift::bench_shift,
};

#[cfg(all(not(target_env = "msvc"), feature = "jemalloc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    #[cfg(feature = "bind-threads")]
    {
        use std::sync::Arc;

        use honeycomb_benches::utils::get_proc_list;
        use hwlocality::{Topology, cpu::binding::CpuBindingFlags};
        use rayon::ThreadPoolBuilder;

        let builder = ThreadPoolBuilder::new();
        let topo = Arc::new(Topology::new().unwrap());
        if let Some(cores) = get_proc_list(&topo) {
            let mut cores = cores.into_iter().cycle();
            builder
                .spawn_handler(|t_builder| {
                    let topo = topo.clone();
                    let core = cores.next().expect("E: unreachable"); // due to cycle

                    std::thread::spawn(move || {
                        // bind
                        let tid = hwlocality::current_thread_id();
                        topo.bind_thread_cpu(tid, &core, CpuBindingFlags::empty())
                            .unwrap();
                        // work
                        t_builder.run();
                    });

                    Ok(())
                })
                .build_global()
                .unwrap();
        } else {
            builder.build_global().unwrap()
        }
    }

    let cli = Cli::parse();

    if cli.simple_precision {
        run_benchmarks::<f32>(cli);
    } else {
        run_benchmarks::<f64>(cli);
    }
}

fn run_benchmarks<T: CoordsFloat>(cli: Cli) {
    prof_init!();

    prof_start!("HCBENCH");
    let map: CMap2<T> = match cli.benches {
        Benches::Grisubal(args) => bench_grisubal(args),
        Benches::Remesh(args) => bench_remesh(args),
        Benches::Shift(args) => bench_shift(args),
    };
    prof_stop!("HCBENCH");

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
