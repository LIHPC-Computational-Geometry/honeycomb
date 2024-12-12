// ------ IMPORTS

use crate::{
    attributes::AttrSparseVec,
    cmap::VertexIdType,
    prelude::{AttributeBind, AttributeUpdate, CMap2, CMapBuilder, Orbit2, OrbitPolicy, Vertex2},
};

// ------ CONTENT

// --- GENERAL

#[test]
fn example_test() {
    // build a triangle
    let mut map: CMap2<f64> = CMapBuilder::default().n_darts(3).build().unwrap();
    map.force_link::<1>(1, 2);
    map.force_link::<1>(2, 3);
    map.force_link::<1>(3, 1);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (1.0, 0.0));
    map.force_write_vertex(3, (0.0, 1.0));

    // checks
    let faces: Vec<_> = map.iter_faces().collect();
    assert_eq!(faces.len(), 1);
    assert_eq!(faces[0], 1);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 1);
    assert_eq!(face.next(), Some(1));
    assert_eq!(face.next(), Some(2));
    assert_eq!(face.next(), Some(3));
    assert_eq!(face.next(), None);

    // build a second triangle
    map.add_free_darts(3);
    map.force_link::<1>(4, 5);
    map.force_link::<1>(5, 6);
    map.force_link::<1>(6, 4);
    map.force_write_vertex(4, (0.0, 2.0));
    map.force_write_vertex(5, (2.0, 0.0));
    map.force_write_vertex(6, (1.0, 1.0));

    // checks
    let faces: Vec<_> = map.iter_faces().collect();
    assert_eq!(&faces, &[1, 4]);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 4);
    assert_eq!(face.next(), Some(4));
    assert_eq!(face.next(), Some(5));
    assert_eq!(face.next(), Some(6));
    assert_eq!(face.next(), None);

    // sew both triangles
    map.force_sew::<2>(2, 4);

    // checks
    assert_eq!(map.beta::<2>(2), 4);
    assert_eq!(map.vertex_id(2), 2);
    assert_eq!(map.vertex_id(5), 2);
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((1.5, 0.0)));
    assert_eq!(map.vertex_id(3), 3);
    assert_eq!(map.vertex_id(4), 3);
    assert_eq!(map.force_read_vertex(3).unwrap(), Vertex2::from((0.0, 1.5)));
    let edges: Vec<_> = map.iter_edges().collect();
    assert_eq!(&edges, &[1, 2, 3, 5, 6]);

    // adjust bottom-right & top-left vertex position
    assert_eq!(
        map.force_write_vertex(2, Vertex2::from((1.0, 0.0))),
        Some(Vertex2::from((1.5, 0.0)))
    );
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((1.0, 0.0)));
    assert_eq!(
        map.force_write_vertex(3, Vertex2::from((0.0, 1.0))),
        Some(Vertex2::from((0.0, 1.5)))
    );
    assert_eq!(map.force_read_vertex(3).unwrap(), Vertex2::from((0.0, 1.0)));

    // separate the diagonal from the rest
    map.force_unsew::<1>(1);
    map.force_unsew::<1>(2);
    map.force_unsew::<1>(6);
    map.force_unsew::<1>(4);
    // break up & remove the diagonal
    map.force_unsew::<2>(2); // this makes dart 2 and 4 free
    map.remove_free_dart(2);
    map.remove_free_dart(4);
    // sew the square back up
    map.force_sew::<1>(1, 5);
    map.force_sew::<1>(6, 3);

    // i-cells
    let faces: Vec<_> = map.iter_faces().collect();
    assert_eq!(&faces, &[1]);
    let edges: Vec<_> = map.iter_edges().collect();
    assert_eq!(&edges, &[1, 3, 5, 6]);
    let vertices: Vec<_> = map.iter_vertices().collect();
    assert_eq!(&vertices, &[1, 3, 5, 6]);
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.force_read_vertex(5).unwrap(), Vertex2::from((1.0, 0.0)));
    assert_eq!(map.force_read_vertex(6).unwrap(), Vertex2::from((1.0, 1.0)));
    assert_eq!(map.force_read_vertex(3).unwrap(), Vertex2::from((0.0, 1.0)));
    // darts
    assert_eq!(map.n_unused_darts(), 2); // there are unused darts since we removed the diagonal
    assert_eq!(map.beta_runtime(1, 1), 5);
    assert_eq!(map.beta_runtime(1, 5), 6);
    assert_eq!(map.beta_runtime(1, 6), 3);
    assert_eq!(map.beta_runtime(1, 3), 1);
}

#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn remove_vertex_twice() {
    // in its default state, all darts/vertices of a map are considered to be used
    let mut map: CMap2<f64> = CMap2::new(4);
    // set vertex 1 as unused
    map.force_remove_vertex(1).unwrap();
    // set vertex 1 as unused, again
    map.force_remove_vertex(1).unwrap(); // this should panic
}

#[test]
#[should_panic(expected = "assertion failed")]
fn remove_dart_twice() {
    // in its default state, all darts/vertices of a map are considered to be used
    // darts are also free
    let mut map: CMap2<f64> = CMap2::new(4);
    // set dart 1 as unused
    map.remove_free_dart(1);
    // set dart 1 as unused, again
    map.remove_free_dart(1); // this should panic
}

// --- (UN)SEW

#[test]
fn two_sew_complete() {
    let mut map: CMap2<f64> = CMap2::new(4);
    map.force_link::<1>(1, 2);
    map.force_link::<1>(3, 4);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_write_vertex(3, (1.0, 1.0));
    map.force_write_vertex(4, (1.0, 0.0));
    map.force_sew::<2>(1, 3);
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.5, 0.0)));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.5, 1.0)));
}

#[test]
fn two_sew_incomplete() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_link::<1>(1, 2);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_write_vertex(3, (1.0, 1.0));
    map.force_sew::<2>(1, 3);
    // missing beta1 image for dart 3
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.5, 1.0)));
    map.force_unsew::<2>(1);
    assert_eq!(map.add_free_dart(), 4);
    map.force_link::<1>(3, 4);
    map.force_sew::<2>(1, 3);
    // missing vertex for dart 4
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.5, 1.0)));
}

#[test]
fn two_sew_no_b1() {
    let mut map: CMap2<f64> = CMap2::new(2);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (1.0, 1.0));
    map.force_sew::<2>(1, 2);
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((1.0, 1.0)));
}

#[test]
// #[should_panic] // FIXME: find a way to test what's printed?
fn two_sew_no_attributes() {
    let mut map: CMap2<f64> = CMap2::new(2);
    map.force_sew::<2>(1, 2); // should panic
}

#[test]
// #[should_panic] // FIXME: find a way to test what's printed?
fn two_sew_no_attributes_bis() {
    let mut map: CMap2<f64> = CMap2::new(4);
    map.force_link::<1>(1, 2);
    map.force_link::<1>(3, 4);
    map.force_sew::<2>(1, 3); // panic
}

#[test]
#[should_panic(expected = "Dart 1 and 3 do not have consistent orientation for 2-sewing")]
fn two_sew_bad_orientation() {
    let mut map: CMap2<f64> = CMap2::new(4);
    map.force_link::<1>(1, 2);
    map.force_link::<1>(3, 4);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0)); // 1->2 goes up
    map.force_write_vertex(3, (1.0, 0.0));
    map.force_write_vertex(4, (1.0, 1.0)); // 3->4 also goes up
    map.force_sew::<2>(1, 3); // panic
}

#[test]
fn one_sew_complete() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_link::<2>(1, 2);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_write_vertex(3, (0.0, 2.0));
    map.force_sew::<1>(1, 3);
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.0, 1.5)));
}

#[test]
fn one_sew_incomplete_attributes() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_link::<2>(1, 2);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_sew::<1>(1, 3);
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.0, 1.0)));
}

#[test]
fn one_sew_incomplete_beta() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_sew::<1>(1, 2);
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.0, 1.0)));
}
#[test]
// #[should_panic] // FIXME: find a way to test what's printed?
fn one_sew_no_attributes() {
    let mut map: CMap2<f64> = CMap2::new(2);
    map.force_sew::<1>(1, 2); // should panic
}

#[test]
// #[should_panic] // FIXME: find a way to test what's printed?
fn one_sew_no_attributes_bis() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_link::<2>(1, 2);
    map.force_sew::<1>(1, 3); // panic
}

// --- IO

#[cfg(feature = "io")]
#[test]
fn io_write() {
    // build a map looking like this:
    //      15
    //     / \
    //    /   \
    //   /     \
    //  16      14
    //  |       |
    //  4---3---7
    //  |   |  /|
    //  |   | / |
    //  |   |/  |
    //  1---2---6
    let mut cmap: CMap2<f32> = CMap2::new(16);
    // bottom left square
    cmap.force_link::<1>(1, 2);
    cmap.force_link::<1>(2, 3);
    cmap.force_link::<1>(3, 4);
    cmap.force_link::<1>(4, 1);
    // bottom right triangles
    cmap.force_link::<1>(5, 6);
    cmap.force_link::<1>(6, 7);
    cmap.force_link::<1>(7, 5);
    cmap.force_link::<2>(7, 8);
    cmap.force_link::<1>(8, 9);
    cmap.force_link::<1>(9, 10);
    cmap.force_link::<1>(10, 8);
    // top polygon
    cmap.force_link::<1>(11, 12);
    cmap.force_link::<1>(12, 13);
    cmap.force_link::<1>(13, 14);
    cmap.force_link::<1>(14, 15);
    cmap.force_link::<1>(15, 16);
    cmap.force_link::<1>(16, 11);
    // assemble
    cmap.force_link::<2>(2, 10);
    cmap.force_link::<2>(3, 11);
    cmap.force_link::<2>(9, 12);

    // insert vertices
    cmap.force_write_vertex(1, (0.0, 0.0));
    cmap.force_write_vertex(2, (1.0, 0.0));
    cmap.force_write_vertex(6, (2.0, 0.0));
    cmap.force_write_vertex(4, (0.0, 1.0));
    cmap.force_write_vertex(3, (1.0, 1.0));
    cmap.force_write_vertex(7, (2.0, 1.0));
    cmap.force_write_vertex(16, (0.0, 2.0));
    cmap.force_write_vertex(15, (1.0, 3.0));
    cmap.force_write_vertex(14, (2.0, 2.0));

    // generate VTK data
    let mut res = String::new();
    cmap.to_vtk_ascii(&mut res);
    println!("{res}");

    // check result
    assert!(res.contains("POINTS 9 float"));
    assert!(res.contains("CELLS 12 44"));
    assert!(res.contains("CELL_TYPES 12"));
    // faces
    assert!(res.contains("4 0 1 2 3"));
    assert!(res.contains("3 1 4 5"));
    assert!(res.contains("4 0 1 2 3"));
    assert!(res.contains("4 0 1 2 3"));
    // edges
    assert!(res.contains("2 0 1"));
    assert!(res.contains("2 3 0"));
    assert!(res.contains("2 1 4"));
    assert!(res.contains("2 4 5"));
    assert!(res.contains("2 5 6"));
    assert!(res.contains("2 6 7"));
    assert!(res.contains("2 7 8"));
    assert!(res.contains("2 8 3"));
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
fn sew_ordering() {
    loom::model(|| {
        // setup the map
        let map: CMap2<f64> = CMapBuilder::default().n_darts(5).build().unwrap();
        map.force_link::<2>(1, 2);
        map.force_link::<1>(4, 5);
        map.force_write_vertex(2, Vertex2(1.0, 1.0));
        map.force_write_vertex(3, Vertex2(1.0, 2.0));
        map.force_write_vertex(5, Vertex2(2.0, 2.0));
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to sew ops:
        // - 1-sew 1 to 3 (t1)
        // - 2-sew 3 to 4 (t2)
        // this will result in a single vertex being define, of ID 2
        // depending on the order of execution of the sews, the value may change
        // 1-sew before 2-sew: (1.5, 1.75)
        // 2-sew before 1-sew: (1.25, 1.5)

        let t1 = loom::thread::spawn(move || {
            m1.force_sew::<1>(1, 3);
        });

        let t2 = loom::thread::spawn(move || {
            m2.force_sew::<2>(3, 4);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // all path should result in the same topological result here
        assert!(arc.force_read_vertex(2).is_some());
        assert!(arc.force_read_vertex(3).is_none());
        assert!(arc.force_read_vertex(5).is_none());
        assert_eq!(Orbit2::new(arc.as_ref(), OrbitPolicy::Vertex, 2).count(), 3);

        // the v2 can have two values though
        let path1 = arc.force_read_vertex(2) == Some(Vertex2(1.5, 1.75));
        let path2 = arc.force_read_vertex(2) == Some(Vertex2(1.25, 1.5));
        assert!(path1 || path2);
    });
}

#[test]
fn unsew_ordering() {
    loom::model(|| {
        // setup the map
        let map: CMap2<f64> = CMapBuilder::default()
            .n_darts(5)
            .add_attribute::<Weight>()
            .build()
            .unwrap();
        map.force_link::<2>(1, 2);
        map.force_link::<2>(3, 4);
        map.force_link::<1>(1, 3);
        map.force_link::<1>(4, 5);
        map.force_write_vertex(2, Vertex2(0.0, 0.0));
        map.force_write_attribute(2, Weight(33));
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to sew ops:
        // - 1-unsew 1 and 3 (t1)
        // - 2-unsew 3 and 4 (t2)
        // this will result in 3 different weights, defined on IDs 2, 3 and 5
        // depending on the order of execution, the final weights will take the following values:
        // 1-unsew before 2-unsew: (W2, W3, W5) = (17, 8, 8)
        // 2-unsew before 1-unsew: (W2, W3, W5) = (9, 8, 16)

        let t1 = loom::thread::spawn(move || {
            m1.force_unsew::<1>(1);
        });

        let t2 = loom::thread::spawn(move || {
            m2.force_unsew::<2>(3);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // all path should result in the same topological result here
        assert!(arc.force_read_attribute::<Weight>(2).is_some());
        assert!(arc.force_read_attribute::<Weight>(3).is_some());
        assert!(arc.force_read_attribute::<Weight>(5).is_some());
        let w2 = arc.force_read_attribute::<Weight>(2).unwrap();
        let w3 = arc.force_read_attribute::<Weight>(3).unwrap();
        let w5 = arc.force_read_attribute::<Weight>(5).unwrap();

        // check scenarios
        let path1 = w2.0 == 17 && w3.0 == 8 && w5.0 == 8;
        let path2 = w2.0 == 9 && w3.0 == 8 && w5.0 == 16;
        assert!(path1 || path2);
    });
}
