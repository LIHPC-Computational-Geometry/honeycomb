use honeycomb_core::{CMap2, DartIdentifier, FloatType, UnsewPolicy};
use honeycomb_utils::generation::square_two_map;

pub fn main() {
    // create a 3x3 grid & remove the central square
    let mut cmap: CMap2<1, FloatType> = square_two_map(3);
    // darts making up the central square
    let (d1, d2, d3, d4): (
        DartIdentifier,
        DartIdentifier,
        DartIdentifier,
        DartIdentifier,
    ) = (17, 18, 19, 20);
    // separate the square from the rest
    cmap.two_unsew(d1, UnsewPolicy::DoNothing);
    cmap.two_unsew(d2, UnsewPolicy::DoNothing);
    cmap.two_unsew(d3, UnsewPolicy::DoNothing);
    cmap.two_unsew(d4, UnsewPolicy::DoNothing);
    // separate dart individually
    cmap.one_unsew(d1, UnsewPolicy::DoNothing);
    cmap.one_unsew(d2, UnsewPolicy::DoNothing);
    cmap.one_unsew(d3, UnsewPolicy::DoNothing);
    cmap.one_unsew(d4, UnsewPolicy::DoNothing);
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
