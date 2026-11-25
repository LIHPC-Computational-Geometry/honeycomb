// src/main.rs
use honeycomb::prelude::*;

pub fn main() {
    // an empty 2D map with 10 darts (plus the null dart, ID 0)
    let map: CMap2<f64> = CMapBuilder::<2>::from_n_darts(6).build().unwrap();
    assert_eq!(map.n_darts(), 7);

    // 3D map equivalent
    let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(6).build().unwrap();
    assert_eq!(map.n_darts(), 7);
}
