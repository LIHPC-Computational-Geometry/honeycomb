use honeycomb_core::prelude::{CMap2, CMapBuilder, DartIdType};

pub fn main() {
    // create a 3x3 grid & remove the central square
    let mut cmap: CMap2<f64> = CMapBuilder::unit_grid(3).build().unwrap();
    // darts making up the central square
    let (d1, d2, d3, d4): (DartIdType, DartIdType, DartIdType, DartIdType) = (17, 18, 19, 20);
    // separate the square from the rest
    cmap.force_unsew::<2>(d1);
    cmap.force_unsew::<2>(d2);
    cmap.force_unsew::<2>(d3);
    cmap.force_unsew::<2>(d4);
    // separate dart individually
    cmap.force_unsew::<1>(d1);
    cmap.force_unsew::<1>(d2);
    cmap.force_unsew::<1>(d3);
    cmap.force_unsew::<1>(d4);
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
