use honeycomb_core::cmap::{CMap2, CMapBuilder};

fn main() {
    let map: CMap2<_> = CMapBuilder::<2, f64>::from_n_darts(4)
        .add_attribute::<Weight>()
        .build()
        .unwrap();

    let _ = map.force_link::<2>(1, 2);
    let _ = map.force_link::<2>(3, 4);
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_write_vertex(3, (1.0, 1.0));
    map.force_write_attribute::<Weight>(2, Weight(5));
    map.force_write_attribute::<Weight>(3, Weight(6));

    let _ = map.force_sew::<1>(1, 3);

    assert_eq!(map.force_read_attribute::<Weight>(2), Some(Weight(11)));

    let _ = map.force_unsew::<1>(1);

    assert_eq!(map.force_read_attribute::<Weight>(2), Some(Weight(6)));
    assert_eq!(map.force_read_attribute::<Weight>(3), Some(Weight(5)));
}
