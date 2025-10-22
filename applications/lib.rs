use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;

use clap::ValueEnum;

#[cfg(feature = "bind-threads")]
use hwlocality::{
    Topology,
    cpu::cpuset::CpuSet,
    object::types::ObjectType,
    topology::support::{DiscoverySupport, FeatureSupport},
};

pub fn hash_file(path: &str) -> Result<u64, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = DefaultHasher::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.write(&buffer[..bytes_read]);
    }

    Ok(hasher.finish())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Backend {
    RayonIter,
    RayonChunks,
    StdThreads,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum FileFormat {
    Cmap,
    Vtk,
}

impl From<Clip> for honeycomb::kernels::grisubal::Clip {
    fn from(value: Clip) -> Self {
        match value {
            Clip::Left => honeycomb::kernels::grisubal::Clip::Left,
            Clip::Right => honeycomb::kernels::grisubal::Clip::Right,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Clip {
    Left,
    Right,
}

pub const NUM_THREADS_VAR: &str = "RAYON_NUM_THREADS";

pub fn get_num_threads() -> Result<usize, String> {
    match std::env::var(NUM_THREADS_VAR) {
        Ok(val) => val.parse::<usize>().map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(feature = "bind-threads")]
#[derive(Debug, Default)]
enum BindingPolicy {
    /// Disable thread binding.
    /// Corresponding `RAYON_PROC_BIND_VALUE`: `false`.
    Disable,
    /// Enable thread binding & prioritize binding of PUs over cores?.
    /// Corresponding `RAYON_PROC_BIND_VALUE`: `close`.
    Close,
    /// Enable thread binding & prioritize binding across cores over filling PUs?.
    /// Corresponding `RAYON_PROC_BIND_VALUE`: `spread`. Default value.
    #[default]
    Spread,
}

/// Environment variable controlling the thread-binding policy.
///
/// The name of this variable and its possible values reflect the OpenMP equivalents.
#[cfg(feature = "bind-threads")]
pub const RAYON_PROC_BIND_VAR: &str = "RAYON_PROC_BIND";

#[cfg(feature = "bind-threads")]
impl BindingPolicy {
    fn from_env() -> Self {
        match std::env::var(RAYON_PROC_BIND_VAR) {
            Ok(val) => match val.to_lowercase().as_str() {
                "false" => Self::Disable,
                "close" => Self::Close,
                "spread" => Self::Spread,
                "" => Self::default(),
                _ => {
                    eprintln!("W: unrecognized RAYON_PROC_BIND value (!= false | close | spread)");
                    eprintln!("   continuing with default (spread)");
                    Self::default()
                }
            },
            Err(e) => {
                match e {
                    std::env::VarError::NotPresent => {}
                    std::env::VarError::NotUnicode(_) => {
                        eprintln!("W: non-unicode RAYON_PROC_BIND value");
                        eprintln!("   continuing with default (spread)");
                    }
                }
                Self::default()
            }
        }
    }
}

// TODO: merge in get_proc_list & define a proper error enum
#[cfg(feature = "bind-threads")]
pub fn check_hwloc_support(topology: &Topology) -> Result<(), String> {
    if !topology.supports(FeatureSupport::discovery, DiscoverySupport::pu_count) {
        return Err("missing PU reporting support".to_string());
    }
    if !topology
        .feature_support()
        .cpu_binding()
        .is_some_and(|s| s.get_thread() && s.set_thread())
    {
        return Err("missing binding support".to_string());
    }

    Ok(())
}

/// Return a list of bind targets ordered according to the desired policy.
///
/// The desired policy is read from an environment variable (see [`RAYON_PROC_BIND_VAR`]). For details on each policy,
/// see [`BindingPolicy`].
///
/// The returned list is used by iterating over a `cycle`d version of it, which corresponds to a round robin logic.
#[cfg(feature = "bind-threads")]
pub fn get_proc_list(topology: &Topology) -> Option<Vec<CpuSet>> {
    let binding_policy = BindingPolicy::from_env();
    let core_depth = topology
        .depth_or_below_for_type(ObjectType::Core)
        .expect("E: unreachable");

    match binding_policy {
        BindingPolicy::Disable => None,
        BindingPolicy::Close => {
            let mut pu_set = Vec::with_capacity(256);
            topology.objects_at_depth(core_depth).for_each(|c| {
                let target = c.cpuset().unwrap().clone_target();
                let w = target.weight();
                if !(w == Some(1) || w == Some(2)) {
                    panic!()
                }
                target
                    .iter_set()
                    .map(CpuSet::from)
                    .for_each(|t| pu_set.push(t));
            });
            Some(pu_set)
        }
        BindingPolicy::Spread => {
            let mut first_pu_set = Vec::with_capacity(128);
            let mut second_pu_set = Vec::with_capacity(128);
            topology.objects_at_depth(core_depth).for_each(|c| {
                let target = c.cpuset().expect("E: unreachable").clone_target();
                // match required bc some modern CPUs have HT/SMT on only some of their cores :)
                match target.weight() {
                    Some(1) => {
                        // one PU per core -> no HT/SMT
                        first_pu_set.push(target);
                    }
                    Some(2) => {
                        // two PUs per core -> HT/SMT
                        let [first_pu, second_pu]: [CpuSet; 2] = target
                            .iter_set()
                            .map(CpuSet::from)
                            .collect::<Vec<_>>()
                            .try_into()
                            .expect("E: unreachable");
                        first_pu_set.push(first_pu);
                        second_pu_set.push(second_pu);
                    }
                    Some(_) | None => {
                        panic!("E: architecture too cursed")
                    }
                }
            });
            first_pu_set.append(&mut second_pu_set);
            Some(first_pu_set)
        }
    }
}

#[macro_export]
macro_rules! bind_rayon_threads {
    () => {
        #[cfg(feature = "bind-threads")]
        {
            use std::sync::Arc;

            use applications::{check_hwloc_support, get_proc_list};
            use hwlocality::{Topology, cpu::binding::CpuBindingFlags};
            use rayon::ThreadPoolBuilder;

            let builder = ThreadPoolBuilder::new();
            let topo = Arc::new(Topology::new().unwrap());
            if check_hwloc_support(&topo).is_ok()
                && let Some(cores) = get_proc_list(&topo)
            {
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
    };
}

#[cfg(feature = "profiling")]
pub static mut PERF_FIFO: Option<File> = None;

/// Attempt to open a fifo at the path `/tmp/hc_perf_control`.
///
/// This macro doesn't generate any code if the `profiling` feature is disabled.
#[macro_export]
macro_rules! prof_init {
    () => {
        #[cfg(feature = "profiling")]
        {
            unsafe {
                $crate::PERF_FIFO = Some(
                    std::fs::OpenOptions::new()
                        .write(true)
                        .open("/tmp/hc_perf_control")
                        .expect("Failed to open FIFO"),
                );
            }
        }
    };
}

/// Write to the `/tmp/hc_perf_control` to enable perf sampling if `${$var}` is defined.
///
/// This macro doesn't generate any code if the `profiling` feature is disabled.
#[macro_export]
macro_rules! prof_start {
    ($var: literal) => {
        #[cfg(feature = "profiling")]
        {
            // use an env variable to select profiled section
            if std::env::var_os($var).is_some() {
                use std::io::Write;
                unsafe {
                    if let Some(ref mut f) = $crate::PERF_FIFO {
                        f.write_all(b"enable\n")
                            .expect("E: failed to write to FIFO");
                        f.flush().expect("E: failed to flush FIFO");
                    }
                }
            }
        }
    };
}

/// Write to the `/tmp/hc_perf_control` to disable perf sampling if `${$var}` is defined.
///
/// This macro doesn't generate any code if the `profiling` feature is disabled.
#[macro_export]
macro_rules! prof_stop {
    ($var: literal) => {
        #[cfg(feature = "profiling")]
        {
            // use an env variable to select profiled section
            if std::env::var_os($var).is_some() {
                use std::io::Write;
                unsafe {
                    if let Some(ref mut f) = $crate::PERF_FIFO {
                        f.write_all(b"disable\n")
                            .expect("E: failed to write to FIFO");
                        f.flush().expect("E: failed to flush FIFO");
                    }
                }
            }
        }
    };
}
