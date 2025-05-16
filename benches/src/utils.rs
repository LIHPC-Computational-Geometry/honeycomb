#[cfg(feature = "thread-binding")]
use std::collections::VecDeque;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;

#[cfg(feature = "thread-binding")]
use hwlocality::{
    Topology,
    object::{TopologyObject, types::ObjectType},
    topology::support::{DiscoverySupport, FeatureSupport},
};

cfg_if::cfg_if! {
    if #[cfg(feature = "_single_precision")] {
        /// Floating-point type alias.
        ///
        /// This is mostly used to run tests using both `f64` and `f32`.
        pub type FloatType = f32;
    } else {
        /// Floating-point type alias.
        ///
        /// This is mostly used to run tests using both `f64` and `f32`.
        pub type FloatType = f64;
    }
}

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

#[cfg(feature = "thread-binding")]
pub fn get_physical_cores<'a, 'b>(
    topology: &'a Topology,
) -> Result<VecDeque<&'b TopologyObject>, String>
where
    'a: 'b,
{
    if topology.supports(FeatureSupport::discovery, DiscoverySupport::pu_count) {
        let cpu_bind_feats = topology
            .feature_support()
            .cpu_binding()
            .is_some_and(|s| s.get_thread() && s.set_thread());
        if cpu_bind_feats {
            // configure the global thread pool
            let core_depth = topology
                .depth_or_below_for_type(ObjectType::Core)
                .map_err(|e| e.to_string())?;
            let cores = topology
                .objects_at_depth(core_depth)
                .collect::<VecDeque<_>>();
            Ok(cores)
        } else {
            Err(
                "Missing CPU binding support; proceeding with the default rayon threadpool"
                    .to_string(),
            )
        }
    } else {
        Err(
            "Missing PU reporting support; proceeding with the default rayon threadpool"
                .to_string(),
        )
    }
}
