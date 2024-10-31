//! Internal timers
//!
//! **This code is only compiled if the `profiling` feature is enabled.**

/// Global timers for execution times per-section.
#[cfg(feature = "profiling")]
pub(crate) static mut TIMERS: [Option<std::time::Duration>; 13] = [None; 13];

/// Kernel section.
#[cfg(feature = "profiling")]
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

#[cfg(feature = "profiling")]
macro_rules! unsafe_time_section {
    ($inst: ident, $sec: expr) => {
        unsafe {
            TIMERS[$sec as usize] = Some($inst.elapsed());
            $inst = std::time::Instant::now();
        }
    };
}

pub(crate) use unsafe_time_section;
