use honeycomb_core::{CoordsFloat, Vertex2};

pub struct IntermediateFace<T: CoordsFloat, I: Iterator<Item = Vertex2<T>>> {
    vertices: I,
    n_vertices: usize,
    center: Vertex2<T>,
}

impl<T: CoordsFloat, I: Iterator<Item = Vertex2<T>> + Clone> IntermediateFace<T, I> {
    pub fn new(it: I) -> Self {
        let n_vertices = it.clone().count();
        let center = Vertex2::from(
            it.clone()
                .map(|vertex2: Vertex2<T>| vertex2.into_inner())
                .reduce(|coords1, coords2| coords1 + coords2)
                .unwrap()
                / T::from(n_vertices).unwrap(),
        );
        Self {
            n_vertices,
            vertices: it,
            center,
        }
    }
}
