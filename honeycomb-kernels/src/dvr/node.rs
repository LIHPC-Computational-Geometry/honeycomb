use enum_dispatch::enum_dispatch;
use honeycomb_core::{
    cmap::{CMap2, DartIdentifier, Orbit2, OrbitPolicy, VertexIdentifier, NULL_DART_ID},
    prelude::{CoordsFloat, Vector2, Vertex2},
};

use super::quality;

/// Node defined by a vertex and its incident 2-cells.
pub struct Node {
    pub id: VertexIdentifier,
    cuts: Vec<Cut>,
}

impl Node {
    pub fn new<T: CoordsFloat>(id: VertexIdentifier, map: &CMap2<T>) -> Self {
        let cuts = Orbit2::new(map, OrbitPolicy::Vertex, id as DartIdentifier)
            .map(|dart| {
                let mut tmp = vec![];
                let mut crt = map.beta::<1>(dart);
                while (crt != dart) || (crt != NULL_DART_ID) {
                    tmp.push(map.vertex_id(crt));
                    crt = map.beta::<1>(dart)
                }
                match tmp.len() {
                    // TODO: raise a proper error
                    0 | 1 => panic!(),
                    // 2 is a tri because we don't count the center vertex
                    2 => CutTri(tmp.try_into().unwrap()).into(),
                    // ditto
                    3 => CutQuad(tmp.try_into().unwrap()).into(),
                    _ => CutAny(tmp).into(),
                }
            })
            .collect();

        Self { id, cuts }
    }

    /// Compute an upper bound for displacement length, independently from the shift's direction.
    pub fn lambda_max<T: CoordsFloat>(&self, map: &CMap2<T>) -> T {
        let v_cent = map.vertex(self.id).unwrap();
        Orbit2::new(map, OrbitPolicy::Vertex, self.id as DartIdentifier)
            .map(|dart| {
                let v_neigh = map.vertex(map.vertex_id(map.beta::<2>(dart))).unwrap();
                (v_cent - v_neigh).norm()
            })
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap()
    }

    /// Compute the quality of the node.
    ///
    /// The original paper provides two definitions for quality:
    /// -
    /// -
    ///
    /// Control over definition usage is done through the boolean argument, `l2_mean`.
    pub fn quality<T: CoordsFloat>(&self, map: &CMap2<T>, shift: &Vector2<T>, l2_mean: bool) -> T {
        let v = map.vertex(self.id).unwrap() + *shift;
        if l2_mean {
            self.cuts
                .iter()
                .map(|cut| cut.quality(map, &v))
                .map(|q| q.powi(2))
                .fold(T::zero(), |q1, q2| q1 + q2)
                .sqrt()
        } else {
            self.cuts
                .iter()
                .map(|cut| cut.quality(map, &v))
                .min_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap()
        }
    }
}

// crate shenanigans to go full functional programming while avoiding dyn objects collection

#[enum_dispatch]
pub trait CutQuality {
    fn quality<T: CoordsFloat>(&self, map: &CMap2<T>, v: &Vertex2<T>) -> T;
}

#[enum_dispatch(CutQuality)]
pub enum Cut {
    CutTri,
    CutQuad,
    CutAny,
}

/// Triangular node cell.
pub struct CutTri([VertexIdentifier; 2]);

impl CutQuality for CutTri {
    fn quality<T: CoordsFloat>(&self, map: &CMap2<T>, v: &Vertex2<T>) -> T {
        let [v1, v2] = [
            map.vertex(self.0[0]).unwrap(),
            map.vertex(self.0[1]).unwrap(),
        ];
        quality::quality_tri(v, &v1, &v2)
    }
}

/// Quadangular node cell.
pub struct CutQuad([VertexIdentifier; 3]);

impl CutQuality for CutQuad {
    fn quality<T: CoordsFloat>(&self, map: &CMap2<T>, v: &Vertex2<T>) -> T {
        // TODO: implement a proper hardcoded version for quads
        let sl = [
            *v,
            map.vertex(self.0[0]).unwrap(),
            map.vertex(self.0[1]).unwrap(),
            map.vertex(self.0[2]).unwrap(),
        ];
        quality::quality_any(&sl)
    }
}

/// N-polygonal node cell, `n >= 5`.
pub struct CutAny(Vec<VertexIdentifier>);

impl CutQuality for CutAny {
    fn quality<T: CoordsFloat>(&self, map: &CMap2<T>, v: &Vertex2<T>) -> T {
        let mut vertices = vec![*v];
        vertices.extend(self.0.iter().map(|vid| map.vertex(*vid).unwrap()));
        quality::quality_any(&vertices)
    }
}
