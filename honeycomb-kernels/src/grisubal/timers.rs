//! Internal timers
//!
//! **This code is only compiled if the `profiling` feature is enabled.**

#[cfg(feature = "profiling")]
/// Global timers for execution times per-section.
pub(crate) static mut TIMERS: [Option<std::time::Duration>; 13] = [None; 13];

#[cfg(feature = "profiling")]
/// Kernel section.
pub(crate) enum Section {
    ImportVTK = 0,
    BuildGeometry,
    DetectOrientation,
    ComputeOverlappingGrid,
    RemoveRedundantPoi,
    BuildMeshTot,
    BuildMeshInit,
    BuildMeshIntersecData,
    BuildMeshInsertIntersec,
    BuildMeshEdgeData,
    BuildMeshInsertEdge,
    Clip,
    Cleanup,
}

macro_rules! start_timer {
    ($inst: ident) => {
        #[cfg(feature = "profiling")]
        let mut $inst = std::time::Instant::now();
    };
}

pub(crate) use start_timer;

macro_rules! unsafe_time_section {
    ($inst: ident, $sec: expr) => {
        #[allow(unused_assignments)]
        #[cfg(feature = "profiling")]
        unsafe {
            timers::TIMERS[$sec as usize] = Some($inst.elapsed());
            $inst = std::time::Instant::now();
        }
    };
}

pub(crate) use unsafe_time_section;

macro_rules! finish {
    ($inst: ident) => {
        #[cfg(feature = "profiling")]
        unsafe {
            timers::TIMERS[timers::Section::Cleanup as usize] = Some($inst.elapsed());
            println!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                timers::TIMERS[0].unwrap().as_nanos(),
                timers::TIMERS[1].unwrap().as_nanos(),
                timers::TIMERS[2].unwrap().as_nanos(),
                timers::TIMERS[3].unwrap().as_nanos(),
                timers::TIMERS[4].unwrap().as_nanos(),
                timers::TIMERS[5].unwrap().as_nanos(),
                timers::TIMERS[6].unwrap().as_nanos(),
                timers::TIMERS[7].unwrap().as_nanos(),
                timers::TIMERS[8].unwrap().as_nanos(),
                timers::TIMERS[9].unwrap().as_nanos(),
                timers::TIMERS[10].unwrap().as_nanos(),
                timers::TIMERS[11].unwrap().as_nanos(),
                timers::TIMERS[12].unwrap().as_nanos(),
            );
        }
    };
}

pub(crate) use finish;
