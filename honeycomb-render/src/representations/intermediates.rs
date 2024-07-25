use honeycomb_core::{CoordsFloat, Vertex2};

pub enum Entity {
    Dart,
    Beta,
    Face,
}

pub struct IntermediateFace<T: CoordsFloat> {
    pub vertices: Vec<Vertex2<T>>,
    pub n_vertices: usize,
    pub center: Vertex2<T>,
}

impl<T: CoordsFloat> IntermediateFace<T> {
    pub fn new<I: Iterator<Item = Vertex2<T>>>(it: I) -> Self {
        let tmp: Vec<Vertex2<T>> = it.collect();
        let n_vertices = tmp.len();
        let center = match n_vertices {
            // with 0 or 1 vertex, there won't be much to render, we can throw in a dummy value
            0 | 1 => Vertex2::from((T::zero(), T::zero())),
            // otherwise, compute the average of all vertices to get the center of the cell
            _ => {
                let Vertex2(x, y) = tmp.iter().fold(Vertex2(T::zero(), T::zero()), |v1, v2| {
                    Vertex2(v1.0 + v2.0, v1.1 + v2.1)
                });
                Vertex2(
                    x / T::from(n_vertices).unwrap(),
                    y / T::from(n_vertices).unwrap(),
                )
            }
        };
        Self {
            n_vertices,
            vertices: tmp,
            center,
        }
    }
}
