use honeycomb_core::{
    cmap::{CMap2, CMapBuilder, DartIdType}
    stm::atomically_with_err,
};

const DIM_GRID: usize = 256;
const N_THREADS: usize = 8;

fn main() {
    let mut map: CMap2<_> = CMapBuilder::<2, f64>::unit_grid(DIM_GRID).build().unwrap();

    // build individual work units
    let faces = map.iter_faces().collect::<Vec<_>>();
    let nd = map.add_free_darts(faces.len() * 2);
    let nd_range = (nd..nd + (faces.len() * 2) as DartIdType).collect::<Vec<_>>();
    let units = faces
        .into_iter()
        .zip(nd_range.chunks(2))
        .collect::<Vec<_>>();

    std::thread::scope(|s| {
        // create batches & move a copy to dispatched thread
        let batches = units.chunks(1 + units.len() / N_THREADS);
        for b in batches {
            s.spawn(|| {
                let locb = b.to_vec();
                locb.into_iter().for_each(|(df, sl)| {
                    let square = df as DartIdType;
                    let &[dsplit1, dsplit2] = sl else {
                        unreachable!()
                    };
                    // we know dart numbering since we constructed a regular grid
                    let (ddown, dright, dup, dleft) = (square, square + 1, square + 2, square + 3);
                    let (dbefore1, dbefore2, dafter1, dafter2) = (ddown, dup, dleft, dright);

                    let _ = map.force_link::<2>(dsplit1, dsplit2); // infallible

                    // internal (un)sews can fail, so we retry until success
                    while atomically_with_err(|trans| {
                        map.unsew::<1>(trans, dbefore1)?;
                        map.unsew::<1>(trans, dbefore2)?;
                        map.sew::<1>(trans, dsplit1, dafter1)?;
                        map.sew::<1>(trans, dsplit2, dafter2)?;

                        map.sew::<1>(trans, dbefore1, dsplit1)?;
                        map.sew::<1>(trans, dbefore2, dsplit2)?;
                        Ok(())
                    })
                    .is_err()
                    {}
                });
            });
        }
    });

    std::hint::black_box(map);
}
