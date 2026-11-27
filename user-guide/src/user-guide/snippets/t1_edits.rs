use honeycomb::prelude::*;

pub fn main() {
    let mut map: CMap2<f64> = CMapBuilder::<2>::from_n_darts(3).build().unwrap();

    // build a first triangle
    map.link::<1>(1, 2).unwrap(); // beta1(1) = 2 & beta0(2) = 1
    map.link::<1>(2, 3).unwrap();
    map.link::<1>(3, 1).unwrap();
    map.write_vertex(1, (0.0, 0.0));
    map.write_vertex(2, (1.0, 0.0));
    map.write_vertex(3, (0.0, 1.0));

    // add 3 darts and build a second triangle
    let first_new = map.allocate_used_darts(3); // returns 4
    assert_eq!(first_new, 4);
    map.link::<1>(4, 5).unwrap();
    map.link::<1>(5, 6).unwrap();
    map.link::<1>(6, 4).unwrap();
    map.write_vertex(4, (0.0, 2.0));
    map.write_vertex(5, (2.0, 0.0));
    map.write_vertex(6, (1.0, 1.0));

    // sew both triangles along the edge made of darts 2 and 4
    map.sew::<2>(2, 4).unwrap();
    let edges: Vec<EdgeIdType> = map.iter_edges().collect();
    assert_eq!(edges.len(), 5);
}
