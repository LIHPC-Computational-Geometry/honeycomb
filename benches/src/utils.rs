use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;

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

#[cfg(feature = "profiling")]
static mut PERF_FIFO: Option<File> = None;

/// Attempt to open a fifo at the path `/tmp/hc_perf_control`.
///
/// This macro doesn't generate any code if the `profiling` feature is disabled.
#[macro_export]
macro_rules! prof_init {
    () => {
        #[cfg(feature = "profiling")]
        {
            unsafe {
                crate::utils::PERF_FIFO = Some(
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
                    if let Some(ref mut f) = crate::utils::PERF_FIFO {
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
                    if let Some(ref mut f) = crate::utils::PERF_FIFO {
                        fifo.write_all(b"disable\n")
                            .expect("E: failed to write to FIFO");
                        fifo.flush().expect("E: failed to flush FIFO");
                    }
                }
            }
        }
    };
}
