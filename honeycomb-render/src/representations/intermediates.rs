use honeycomb_core::{CoordsFloat, Vertex2};

pub struct IntermediateFace<T: CoordsFloat> {
    pub vertices: Vec<Vertex2<T>>,
    pub n_vertices: usize,
    pub center: Vertex2<T>,
}

impl<T: CoordsFloat> IntermediateFace<T> {
    pub fn new<I: Iterator<Item = Vertex2<T>>>(it: I) -> Self {
        let tmp: Vec<Vertex2<T>> = it.collect();
        let n_vertices = tmp.len();
        let center = Vertex2::from(
            tmp.iter()
                .map(|vertex2: &Vertex2<T>| vertex2.into_inner())
                .reduce(|coords1, coords2| coords1 + coords2)
                .unwrap()
                / T::from(n_vertices).unwrap(),
        );
        Self {
            n_vertices,
            vertices: tmp,
            center,
        }
    }
}
