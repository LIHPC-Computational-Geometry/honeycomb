// ------ IMPORTS

use crate::{
    attributes::{AttrSparseVec, AttributeBind, AttributeUpdate},
    cmap::{CMap3, Orbit3, OrbitPolicy, VertexIdType},
    geometry::Vertex3,
};

// ------ CONTENT

#[test]
fn remove_vertex_twice() {
    let map: CMap3<f64> = CMap3::new(4);
    assert!(map.force_write_vertex(1, (1.0, 1.0, 1.0)).is_none());
    assert_eq!(map.force_remove_vertex(1), Some(Vertex3(1.0, 1.0, 1.0)));
    assert!(map.force_remove_vertex(1).is_none());
}

#[test]
#[should_panic(expected = "assertion failed")]
fn remove_dart_twice() {
    // in its default state, all darts are:
    // - used
    // - free
    let mut map: CMap3<f64> = CMap3::new(4);
    map.remove_free_dart(1);
    map.remove_free_dart(1); // this should panic
}

// --- (UN)SEW

#[test]
fn three_sew_complete() {
    let map: CMap3<f64> = CMap3::new(8);
    map.force_link::<1>(1, 2);
    map.force_link::<1>(2, 3);
    map.force_link::<1>(3, 4);
    map.force_link::<1>(4, 1);
    map.force_write_vertex(1, Vertex3(0.0, 0.0, 0.0));
    map.force_write_vertex(2, Vertex3(1.0, 0.0, 0.0));
    map.force_write_vertex(3, Vertex3(1.0, 1.0, 0.0));
    map.force_write_vertex(4, Vertex3(0.0, 1.0, 0.0));

    map.force_link::<1>(5, 6);
    map.force_link::<1>(6, 7);
    map.force_link::<1>(7, 8);
    map.force_link::<1>(8, 5);
    map.force_write_vertex(5, Vertex3(0.0, 0.0, 1.0));
    map.force_write_vertex(6, Vertex3(0.0, 1.0, 1.0));
    map.force_write_vertex(7, Vertex3(1.0, 1.0, 1.0));
    map.force_write_vertex(8, Vertex3(1.0, 0.0, 1.0));

    map.force_sew::<3>(1, 8);
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex3(0.0, 0.0, 0.5));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex3(1.0, 0.0, 0.5));
    assert_eq!(map.force_read_vertex(3).unwrap(), Vertex3(1.0, 1.0, 0.5));
    assert_eq!(map.force_read_vertex(4).unwrap(), Vertex3(0.0, 1.0, 0.5));
}

#[test]
#[should_panic(expected = "Dart 1 and 3 do not have consistent orientation for 3-sewing")]
fn three_sew_bad_orientation_3d() {
    let map: CMap3<f64> = CMap3::new(8);
    map.force_link::<1>(1, 2);
    map.force_link::<1>(2, 3);
    map.force_link::<1>(3, 4);
    map.force_link::<1>(4, 1);
    map.force_link::<1>(5, 6);
    map.force_link::<1>(6, 7);
    map.force_link::<1>(7, 8);
    map.force_link::<1>(8, 5);
    map.force_write_vertex(1, Vertex3(0.0, 0.0, 0.0));
    map.force_write_vertex(2, Vertex3(1.0, 0.0, 0.0));
    map.force_write_vertex(3, Vertex3(1.0, 1.0, 0.0));
    map.force_write_vertex(4, Vertex3(0.0, 1.0, 0.0));
    map.force_write_vertex(5, Vertex3(0.0, 0.0, 1.0));
    map.force_write_vertex(6, Vertex3(1.0, 0.0, 1.0));
    map.force_write_vertex(7, Vertex3(1.0, 1.0, 1.0));
    map.force_write_vertex(8, Vertex3(0.0, 1.0, 1.0));
    map.force_sew::<3>(1, 5); // panic due to inconsistent orientation
}

// --- PARALLEL

#[derive(Debug, Clone, Copy, Default)]
struct Weight(pub u32);

impl AttributeUpdate for Weight {
    fn merge(attr1: Self, attr2: Self) -> Self {
        Self(attr1.0 + attr2.0)
    }

    fn split(attr: Self) -> (Self, Self) {
        // adding the % to keep things conservative
        (Weight(attr.0 / 2 + attr.0 % 2), Weight(attr.0 / 2))
    }
}

impl AttributeBind for Weight {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}

#[test]
fn sew_ordering_3d() {
    loom::model(|| {
        // setup the map
        let map: CMap3<f64> = CMap3::new(5);
        map.force_link::<2>(1, 2);
        map.force_link::<1>(4, 5);
        map.force_write_vertex(2, Vertex3(1.0, 1.0, 1.0));
        map.force_write_vertex(3, Vertex3(1.0, 2.0, 1.0));
        map.force_write_vertex(5, Vertex3(2.0, 2.0, 1.0));
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to sew ops:
        // - 1-sew 1 to 3 (t1)
        // - 2-sew 3 to 4 (t2)
        // this will result in a single vertex being defined, of ID 2
        // depending on the order of execution of the sews, the value may change

        let t1 = loom::thread::spawn(move || {
            m1.force_sew::<1>(1, 3);
        });

        let t2 = loom::thread::spawn(move || {
            m2.force_sew::<2>(3, 4);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // all paths should result in the same topological result here
        assert!(arc.force_read_vertex(2).is_some());
        assert!(arc.force_read_vertex(3).is_none());
        assert!(arc.force_read_vertex(5).is_none());
        assert_eq!(Orbit3::new(arc.as_ref(), OrbitPolicy::Vertex, 2).count(), 3);

        // the vertex can have two values though; we don't check for exact values here
        assert!(arc.force_read_vertex(2).is_some());
    });
}

/*
#[test]
fn unsew_ordering_3d() {
    loom::model(|| {
        // setup the map
        let map: CMap3<f64> = CMapBuilder::default()
            .n_darts(5)
            .add_attribute::<Weight>()
            .build()
            .unwrap();
        map.force_link::<2>(1, 2);
        map.force_link::<2>(3, 4);
        map.force_link::<1>(1, 3);
        map.force_link::<1>(4, 5);
        map.force_write_vertex(2, Vertex3(0.0, 0.0, 0.0));
        map.force_write_attribute(2, Weight(33));
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to unsew ops:
        // - 1-unsew 1 and 3 (t1)
        // - 2-unsew 3 and 4 (t2)
        // this will result in different weights, defined on IDs 2, 3, and 5

        let t1 = loom::thread::spawn(move || {
            m1.force_unsew::<1>(1);
        });

        let t2 = loom::thread::spawn(move || {
            m2.force_unsew::<2>(3);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // all paths should result in the same topological result here
        assert!(arc.force_read_attribute::<Weight>(2).is_some());
        assert!(arc.force_read_attribute::<Weight>(3).is_some());
        assert!(arc.force_read_attribute::<Weight>(5).is_some());
        let _w2 = arc.force_read_attribute::<Weight>(2).unwrap();
        let _w3 = arc.force_read_attribute::<Weight>(3).unwrap();
        let _w5 = arc.force_read_attribute::<Weight>(5).unwrap();

        // We don't check for exact values here as they might differ based on execution order
    });
}
*/
