use honeycomb_core::{utils::GridBuilder, CMap2, DartIdentifier, FloatType};

pub fn main() {
    // create a 3x3 grid & remove the central square
    let mut cmap: CMap2<FloatType> = GridBuilder::unit_squares(3).build2();
    // darts making up the central square
    let (d1, d2, d3, d4): (
        DartIdentifier,
        DartIdentifier,
        DartIdentifier,
        DartIdentifier,
    ) = (17, 18, 19, 20);
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
    cmap.used_size("memusage_example");
    cmap.allocated_size("memusage_example");
    cmap.effective_size("memusage_example");
}
