// ------ IMPORTS

use crate::prelude::{CMap2, CMapBuilder};

// ------ CONTENT

#[test]
fn example_test() {
    let builder = CMapBuilder::default().n_darts(10);
    let cmap: CMap2<f64> = builder.build().unwrap();
    assert_eq!(cmap.n_darts(), 11);
}
