use honeycomb::prelude::*;

// user-defined struct and trait implementation
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Weight(pub u32);

impl AttributeUpdate for Weight {
    /* ... */
}

impl AttributeBind for Weight {
    /* ... */
}

pub fn main() {
    let map: CMap2<f64> = CMapBuilder::<2>::from_n_darts(4)
        .add_attribute::<Weight>()
        .build()
        .unwrap();

    map.write_attribute::<Weight>(2, Weight(5));
    assert_eq!(map.read_attribute::<Weight>(2), Some(Weight(5)));

    // sew / unsew will automatically merge / split the attributes of the sewn cells
}
