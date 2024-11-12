use honeycomb_core::prelude::{CMap2, CMapBuilder, DartId};

pub fn main() {
    // create a 3x3 grid & remove the central square
    let mut cmap: CMap2<f64> = CMapBuilder::unit_grid(3).build().unwrap();
    // darts making up the central square
    let (d1, d2, d3, d4) = (DartId(17), DartId(18), DartId(19), DartId(20));
    // separate the square from the rest
    cmap.two_unsew(d1);
    cmap.two_unsew(d2);
    cmap.two_unsew(d3);
    cmap.two_unsew(d4);
    // separate dart individually
    cmap.one_unsew(d1);
    cmap.one_unsew(d2);
    cmap.one_unsew(d3);
    cmap.one_unsew(d4);
    // remove darts
    cmap.remove_free_dart(d1);
    cmap.remove_free_dart(d2);
    cmap.remove_free_dart(d3);
    cmap.remove_free_dart(d4);
    // dump memory usage
    cmap.used_size("memusage_example").unwrap();
    cmap.allocated_size("memusage_example").unwrap();
    cmap.effective_size("memusage_example").unwrap();
}
