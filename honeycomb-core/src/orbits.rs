use crate::{CoordsFloat, TwoMap};

pub struct Orbit<'a, const N_MARKS: usize, T: CoordsFloat> {
    map_handle: &'a TwoMap<N_MARKS, T>,
}
