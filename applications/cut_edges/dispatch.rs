use honeycomb::{
    prelude::{
        CMap2, CoordsFloat, DartIdType, EdgeIdType, SewError,
        remeshing::{cut_inner_edge, cut_outer_edge},
    },
    stm::{Transaction, TransactionControl, TransactionResult},
};
use rayon::prelude::*;

#[inline]
pub fn dispatch_rayon<T: CoordsFloat>(
    map: &CMap2<T>,
    edges: &mut Vec<EdgeIdType>,
    darts: &[DartIdType],
) -> u32 {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();
    units
        .into_par_iter()
        .map(|(e, new_darts)| {
            let mut n_retry = 0;
            if map.is_i_free::<2>(e as DartIdType) {
                while !process_outer_edge(map, &mut n_retry, e, new_darts).is_validated() {}
            } else {
                while !process_inner_edge(map, &mut n_retry, e, new_darts).is_validated() {}
            }
            n_retry as u32
        }) // par_map
        .sum()
}

#[inline]
pub fn dispatch_rayon_chunks<T: CoordsFloat>(
    map: &CMap2<T>,
    edges: &mut Vec<EdgeIdType>,
    darts: &[DartIdType],
    n_threads: usize,
) -> u32 {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();
    units
        .par_chunks(1 + units.len() / n_threads)
        .map(|c| {
            let mut n = 0;
            c.iter().for_each(|&(e, new_darts)| {
                let mut n_retry = 0;
                if map.is_i_free::<2>(e as DartIdType) {
                    while !process_outer_edge(map, &mut n_retry, e, new_darts).is_validated() {}
                } else {
                    while !process_inner_edge(map, &mut n_retry, e, new_darts).is_validated() {}
                }
                n += n_retry as u32;
            });
            n
        }) // par_for_each
        .sum()
}

#[inline]
pub fn dispatch_std_threads<T: CoordsFloat>(
    map: &CMap2<T>,
    edges: &mut Vec<EdgeIdType>,
    darts: &[DartIdType],
    n_threads: usize,
) -> u32 {
    let units: Vec<(u32, [u32; 6])> = edges
        .drain(..)
        .zip(darts.chunks(6))
        .map(|(e, sl)| (e, sl.try_into().unwrap()))
        .collect();

    #[cfg(feature = "bind-threads")]
    {
        use std::sync::Arc;

        use hwlocality::{Topology, cpu::binding::CpuBindingFlags};

        use applications::get_proc_list;

        let topo = Arc::new(Topology::new().unwrap());
        let mut cores = get_proc_list(&topo).unwrap_or_default().into_iter().cycle();
        std::thread::scope(|s| {
            let mut handles = Vec::new();
            for wl in units.chunks(1 + units.len() / n_threads) {
                let topo = topo.clone();
                let core = cores.next();
                handles.push(s.spawn(move || {
                    // bind
                    if let Some(c) = core {
                        let tid = hwlocality::current_thread_id();
                        topo.bind_thread_cpu(tid, &c, CpuBindingFlags::empty())
                            .unwrap();
                    }
                    // work
                    let mut n = 0;
                    wl.iter().for_each(|&(e, new_darts)| {
                        let mut n_retry = 0;
                        if map.is_i_free::<2>(e as DartIdType) {
                            while !process_outer_edge(map, &mut n_retry, e, new_darts)
                                .is_validated()
                            {}
                        } else {
                            while !process_inner_edge(map, &mut n_retry, e, new_darts)
                                .is_validated()
                            {}
                        }
                        n += n_retry as u32;
                    });
                    n
                })); // s.spawn
            } // for wl in workloads
            handles.into_iter().map(|h| h.join().unwrap()).sum()
        }) // std::thread::scope
    }

    #[cfg(not(feature = "bind-threads"))]
    {
        std::thread::scope(|s| {
            let mut handles = Vec::new();
            for wl in units.chunks(1 + units.len() / n_threads) {
                handles.push(s.spawn(|| {
                    let mut n = 0;
                    wl.iter().for_each(|&(e, new_darts)| {
                        let mut n_retry = 0;
                        if map.is_i_free::<2>(e as DartIdType) {
                            while !process_outer_edge(map, &mut n_retry, e, new_darts)
                                .is_validated()
                            {}
                        } else {
                            while !process_inner_edge(map, &mut n_retry, e, new_darts)
                                .is_validated()
                            {}
                        }
                        n += n_retry as u32;
                    });
                    n
                })); // s.spawn
            } // for wl in workloads
            handles.into_iter().map(|h| h.join().unwrap()).sum()
        }) // std::thread::scope
    }
}

#[inline]
fn process_outer_edge<T: CoordsFloat>(
    map: &CMap2<T>,
    n_retry: &mut u8,
    e: EdgeIdType,
    [nd1, nd2, nd3, _, _, _]: [DartIdType; 6],
) -> TransactionResult<(), SewError> {
    Transaction::with_control_and_err(
        |_| {
            *n_retry += 1;
            TransactionControl::Retry
        },
        |t| cut_outer_edge(t, map, e, [nd1, nd2, nd3]),
    ) // Transaction::with_control
}

#[inline]
fn process_inner_edge<T: CoordsFloat>(
    map: &CMap2<T>,
    n_retry: &mut u8,
    e: EdgeIdType,
    nds: [DartIdType; 6],
) -> TransactionResult<(), SewError> {
    Transaction::with_control_and_err(
        |_| {
            *n_retry += 1;
            TransactionControl::Retry
        },
        |t| cut_inner_edge(t, map, e, nds),
    ) // Transaction::with_control
}
