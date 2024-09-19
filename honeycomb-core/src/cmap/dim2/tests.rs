// ------ IMPORTS

use crate::prelude::{CMap2, CMapBuilder, Orbit2, OrbitPolicy, Vertex2, NULL_DART_ID};

// ------ CONTENT

// --- GENERAL

#[test]
fn example_test() {
    // build a triangle
    let mut map: CMap2<f64> = CMapBuilder::default().n_darts(3).build().unwrap();
    map.one_link(1, 2);
    map.one_link(2, 3);
    map.one_link(3, 1);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 0.0));
    map.insert_vertex(3, (0.0, 1.0));

    // checks
    let faces = map.fetch_faces();
    assert_eq!(faces.identifiers.len(), 1);
    assert_eq!(faces.identifiers[0], 1);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 1);
    assert_eq!(face.next(), Some(1));
    assert_eq!(face.next(), Some(2));
    assert_eq!(face.next(), Some(3));
    assert_eq!(face.next(), None);

    // build a second triangle
    map.add_free_darts(3);
    map.one_link(4, 5);
    map.one_link(5, 6);
    map.one_link(6, 4);
    map.insert_vertex(4, (0.0, 2.0));
    map.insert_vertex(5, (2.0, 0.0));
    map.insert_vertex(6, (1.0, 1.0));

    // checks
    let faces = map.fetch_faces();
    assert_eq!(&faces.identifiers, &[1, 4]);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 4);
    assert_eq!(face.next(), Some(4));
    assert_eq!(face.next(), Some(5));
    assert_eq!(face.next(), Some(6));
    assert_eq!(face.next(), None);

    // sew both triangles
    map.two_sew(2, 4);

    // checks
    assert_eq!(map.beta::<2>(2), 4);
    assert_eq!(map.vertex_id(2), 2);
    assert_eq!(map.vertex_id(5), 2);
    assert_eq!(map.vertex(2).unwrap(), Vertex2::from((1.5, 0.0)));
    assert_eq!(map.vertex_id(3), 3);
    assert_eq!(map.vertex_id(4), 3);
    assert_eq!(map.vertex(3).unwrap(), Vertex2::from((0.0, 1.5)));
    let edges = map.fetch_edges();
    assert_eq!(&edges.identifiers, &[1, 2, 3, 5, 6]);

    // adjust bottom-right & top-left vertex position
    assert_eq!(
        map.replace_vertex(2, Vertex2::from((1.0, 0.0))),
        Some(Vertex2::from((1.5, 0.0)))
    );
    assert_eq!(map.vertex(2).unwrap(), Vertex2::from((1.0, 0.0)));
    assert_eq!(
        map.replace_vertex(3, Vertex2::from((0.0, 1.0))),
        Some(Vertex2::from((0.0, 1.5)))
    );
    assert_eq!(map.vertex(3).unwrap(), Vertex2::from((0.0, 1.0)));

    // separate the diagonal from the rest
    map.one_unsew(1);
    map.one_unsew(2);
    map.one_unsew(6);
    map.one_unsew(4);
    // break up & remove the diagonal
    map.two_unsew(2); // this makes dart 2 and 4 free
    map.remove_free_dart(2);
    map.remove_free_dart(4);
    // sew the square back up
    map.one_sew(1, 5);
    map.one_sew(6, 3);

    // i-cells
    let faces = map.fetch_faces();
    assert_eq!(&faces.identifiers, &[1]);
    let edges = map.fetch_edges();
    assert_eq!(&edges.identifiers, &[1, 3, 5, 6]);
    let vertices = map.fetch_vertices();
    assert_eq!(&vertices.identifiers, &[1, 3, 5, 6]);
    assert_eq!(map.vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.vertex(5).unwrap(), Vertex2::from((1.0, 0.0)));
    assert_eq!(map.vertex(6).unwrap(), Vertex2::from((1.0, 1.0)));
    assert_eq!(map.vertex(3).unwrap(), Vertex2::from((0.0, 1.0)));
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
    map.remove_vertex(1).unwrap();
    // set vertex 1 as unused, again
    map.remove_vertex(1).unwrap(); // this should panic
}

#[test]
#[should_panic(expected = "assertion failed: self.unused_darts.insert(dart_id)")]
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
    map.one_link(1, 2);
    map.one_link(3, 4);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.insert_vertex(3, (1.0, 1.0));
    map.insert_vertex(4, (1.0, 0.0));
    map.two_sew(1, 3);
    assert_eq!(map.vertex(1).unwrap(), Vertex2::from((0.5, 0.0)));
    assert_eq!(map.vertex(2).unwrap(), Vertex2::from((0.5, 1.0)));
}

#[test]
fn two_sew_incomplete() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.one_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.insert_vertex(3, (1.0, 1.0));
    map.two_sew(1, 3);
    // missing beta1 image for dart 3
    assert_eq!(map.vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.vertex(2).unwrap(), Vertex2::from((0.5, 1.0)));
    map.two_unsew(1);
    assert_eq!(map.add_free_dart(), 4);
    map.one_link(3, 4);
    map.two_sew(1, 3);
    // missing vertex for dart 4
    assert_eq!(map.vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.vertex(2).unwrap(), Vertex2::from((0.5, 1.0)));
}

#[test]
fn two_sew_no_b1() {
    let mut map: CMap2<f64> = CMap2::new(2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 1.0));
    map.two_sew(1, 2);
    assert_eq!(map.vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.vertex(2).unwrap(), Vertex2::from((1.0, 1.0)));
}

#[test]
#[should_panic(
    expected = "No vertices defined on either darts 1/2 , use `two_link` instead of `two_sew`"
)]
fn two_sew_no_attributes() {
    let mut map: CMap2<f64> = CMap2::new(2);
    map.two_sew(1, 2); // should panic
}

#[test]
#[should_panic(expected = "E: attempt to merge two undefined vertices")]
fn two_sew_no_attributes_bis() {
    let mut map: CMap2<f64> = CMap2::new(4);
    map.one_link(1, 2);
    map.one_link(3, 4);
    map.two_sew(1, 3); // panic
}

#[test]
#[should_panic(expected = "Dart 1 and 3 do not have consistent orientation for 2-sewing")]
fn two_sew_bad_orientation() {
    let mut map: CMap2<f64> = CMap2::new(4);
    map.one_link(1, 2);
    map.one_link(3, 4);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0)); // 1->2 goes up
    map.insert_vertex(3, (1.0, 0.0));
    map.insert_vertex(4, (1.0, 1.0)); // 3->4 also goes up
    map.two_sew(1, 3); // panic
}

#[test]
fn one_sew_complete() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.two_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.insert_vertex(3, (0.0, 2.0));
    map.one_sew(1, 3);
    assert_eq!(map.vertex(2).unwrap(), Vertex2::from((0.0, 1.5)));
}

#[test]
fn one_sew_incomplete_attributes() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.two_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.one_sew(1, 3);
    assert_eq!(map.vertex(2).unwrap(), Vertex2::from((0.0, 1.0)));
}

#[test]
fn one_sew_incomplete_beta() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.one_sew(1, 2);
    assert_eq!(map.vertex(2).unwrap(), Vertex2::from((0.0, 1.0)));
}
#[test]
#[should_panic(expected = "No vertex defined on dart 2, use `one_link` instead of `one_sew`")]
fn one_sew_no_attributes() {
    let mut map: CMap2<f64> = CMap2::new(2);
    map.one_sew(1, 2); // should panic
}

#[test]
#[should_panic(expected = "E: attempt to merge two undefined vertices")]
fn one_sew_no_attributes_bis() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.two_link(1, 2);
    map.one_sew(1, 3); // panic
}

// --- ADVANCED

// split_edge

#[test]
fn split_edge_complete() {
    // before
    //    <--6---   <--5---   <--4---
    //  1         2         3         4
    //    ---1-->   ---2-->   ---3-->
    let mut map: CMap2<f64> = CMap2::new(6);
    map.one_link(1, 2);
    map.one_link(2, 3);
    map.one_link(4, 5);
    map.one_link(5, 6);
    map.two_link(1, 6);
    map.two_link(2, 5);
    map.two_link(3, 4);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 0.0));
    map.insert_vertex(3, (2.0, 0.0));
    map.insert_vertex(4, (3.0, 0.0));
    // split
    map.split_edge(2, None);
    // after
    //    <--6---   <8- <5-   <--4---
    //  1         2    7    3         4
    //    ---1-->   -2> -7>   ---3-->
    assert_eq!(map.beta::<2>(2), 8);
    assert_eq!(map.beta::<1>(1), 2);
    assert_eq!(map.beta::<1>(2), 7);
    assert_eq!(map.beta::<1>(7), 3);

    assert_eq!(map.beta::<2>(5), 7);
    assert_eq!(map.beta::<1>(4), 5);
    assert_eq!(map.beta::<1>(5), 8);
    assert_eq!(map.beta::<1>(8), 6);

    assert_eq!(map.vertex_id(8), 7);
    assert_eq!(map.vertex_id(7), 7);

    assert_eq!(map.vertex(2), Some(Vertex2::from((1.0, 0.0))));
    assert_eq!(map.vertex(7), Some(Vertex2::from((1.5, 0.0))));
    assert_eq!(map.vertex(3), Some(Vertex2::from((2.0, 0.0))));
}

#[test]
fn split_edge_isolated() {
    // before
    //    <--2---
    //  1         2
    //    ---1-->
    let mut map: CMap2<f64> = CMap2::new(2);
    map.two_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 0.0));
    // split
    map.split_edge(1, Some(0.6));
    // after
    //    <-4- <2-
    //  1     3    2
    //    -1-> -3>
    assert_eq!(map.beta::<2>(1), 4);
    assert_eq!(map.beta::<1>(1), 3);

    assert_eq!(map.beta::<2>(2), 3);
    assert_eq!(map.beta::<1>(2), 4);

    assert_eq!(map.vertex_id(3), 3);
    assert_eq!(map.vertex_id(4), 3);

    assert_eq!(map.vertex(1), Some(Vertex2::from((0.0, 0.0))));
    assert_eq!(map.vertex(3), Some(Vertex2::from((0.6, 0.0))));
    assert_eq!(map.vertex(2), Some(Vertex2::from((1.0, 0.0))));
}

#[test]
fn split_single_dart() {
    // before
    //  1 -----> 2 ->
    let mut map: CMap2<f64> = CMap2::new(2);
    map.one_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 0.0));
    // split
    map.split_edge(1, None);
    // after
    //  1 -> 3 -> 2 ->
    assert_eq!(map.beta::<1>(1), 3);
    assert_eq!(map.beta::<1>(3), 2);
    assert_eq!(map.beta::<2>(3), NULL_DART_ID);
    assert_eq!(map.vertex(3), Some(Vertex2::from((0.5, 0.0))));
}

#[test]
#[should_panic(expected = "attempt to split an edge that is not fully defined in the first place")]
fn split_edge_missing_vertex() {
    //    <--2---
    //  1         ?
    //    ---1-->
    let mut map: CMap2<f64> = CMap2::new(2);
    map.two_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    // map.insert_vertex(2, (1.0, 0.0)); missing vertex!
    // split
    map.split_edge(1, None);
}

// splitn_edge

#[test]
fn splitn_edge_complete() {
    // before
    //    <--6---   <--5---   <--4---
    //  1         2         3         4
    //    ---1-->   ---2-->   ---3-->
    let mut map: CMap2<f64> = CMap2::new(6);
    map.one_link(1, 2);
    map.one_link(2, 3);
    map.one_link(4, 5);
    map.one_link(5, 6);
    map.two_link(1, 6);
    map.two_link(2, 5);
    map.two_link(3, 4);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 0.0));
    map.insert_vertex(3, (2.0, 0.0));
    map.insert_vertex(4, (3.0, 0.0));
    // split
    let new_darts = map.splitn_edge(2, [0.25, 0.50, 0.75]);
    // after
    //    <--6---             <--4---
    //  1         2 -7-8-9- 3         4
    //    ---1-->             ---3-->
    assert_eq!(&new_darts, &[7, 8, 9]);
    assert_eq!(map.vertex(7), Some(Vertex2(1.25, 0.0)));
    assert_eq!(map.vertex(8), Some(Vertex2(1.50, 0.0)));
    assert_eq!(map.vertex(9), Some(Vertex2(1.75, 0.0)));

    assert_eq!(map.beta::<1>(2), 7);
    assert_eq!(map.beta::<1>(7), 8);
    assert_eq!(map.beta::<1>(8), 9);
    assert_eq!(map.beta::<1>(9), 3);

    assert_eq!(map.beta::<1>(5), 10);
    assert_eq!(map.beta::<1>(10), 11);
    assert_eq!(map.beta::<1>(11), 12);
    assert_eq!(map.beta::<1>(12), 6);

    assert_eq!(map.beta::<2>(2), 12);
    assert_eq!(map.beta::<2>(7), 11);
    assert_eq!(map.beta::<2>(8), 10);
    assert_eq!(map.beta::<2>(9), 5);
}

#[test]
fn splitn_edge_isolated() {
    // before
    //    <--2---
    //  1         2
    //    ---1-->
    let mut map: CMap2<f64> = CMap2::new(2);
    map.two_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 0.0));
    // split
    let new_darts = map.splitn_edge(1, [0.25, 0.50, 0.75]);
    // after
    //    <-<-<-<
    //  1 -3-4-5- 2
    //    >->->->
    assert_eq!(&new_darts, &[3, 4, 5]);
    assert_eq!(map.vertex(3), Some(Vertex2(0.25, 0.0)));
    assert_eq!(map.vertex(4), Some(Vertex2(0.50, 0.0)));
    assert_eq!(map.vertex(5), Some(Vertex2(0.75, 0.0)));

    assert_eq!(map.beta::<1>(1), 3);
    assert_eq!(map.beta::<1>(3), 4);
    assert_eq!(map.beta::<1>(4), 5);
    assert_eq!(map.beta::<1>(5), NULL_DART_ID);

    assert_eq!(map.beta::<1>(2), 6);
    assert_eq!(map.beta::<1>(6), 7);
    assert_eq!(map.beta::<1>(7), 8);
    assert_eq!(map.beta::<1>(8), NULL_DART_ID);

    assert_eq!(map.beta::<2>(1), 8);
    assert_eq!(map.beta::<2>(3), 7);
    assert_eq!(map.beta::<2>(4), 6);
    assert_eq!(map.beta::<2>(5), 2);
}

#[test]
fn splitn_single_dart() {
    // before
    //  1 -----> 2 ->
    let mut map: CMap2<f64> = CMap2::new(2);
    map.one_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 0.0));
    // split
    let new_darts = map.splitn_edge(1, [0.25, 0.50, 0.75]);
    // after
    //  1 -> 3 -> 4 -> 5 -> 2 ->
    assert_eq!(&new_darts, &[3, 4, 5]);
    assert_eq!(map.vertex(3), Some(Vertex2(0.25, 0.0)));
    assert_eq!(map.vertex(4), Some(Vertex2(0.50, 0.0)));
    assert_eq!(map.vertex(5), Some(Vertex2(0.75, 0.0)));

    assert_eq!(map.beta::<1>(1), 3);
    assert_eq!(map.beta::<1>(3), 4);
    assert_eq!(map.beta::<1>(4), 5);
    assert_eq!(map.beta::<1>(5), 2);

    assert_eq!(map.beta::<2>(1), NULL_DART_ID);
    assert_eq!(map.beta::<2>(3), NULL_DART_ID);
    assert_eq!(map.beta::<2>(4), NULL_DART_ID);
    assert_eq!(map.beta::<2>(5), NULL_DART_ID);
}

#[test]
#[should_panic(expected = "attempt to split an edge that is not fully defined in the first place")]
fn splitn_edge_missing_vertex() {
    //    <--2---
    //  1         ?
    //    ---1-->
    let mut map: CMap2<f64> = CMap2::new(2);
    map.two_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    // map.insert_vertex(2, (1.0, 0.0)); missing vertex!
    // split
    map.splitn_edge(1, [0.25, 0.50, 0.75]);
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
    cmap.one_link(1, 2);
    cmap.one_link(2, 3);
    cmap.one_link(3, 4);
    cmap.one_link(4, 1);
    // bottom right triangles
    cmap.one_link(5, 6);
    cmap.one_link(6, 7);
    cmap.one_link(7, 5);
    cmap.two_link(7, 8);
    cmap.one_link(8, 9);
    cmap.one_link(9, 10);
    cmap.one_link(10, 8);
    // top polygon
    cmap.one_link(11, 12);
    cmap.one_link(12, 13);
    cmap.one_link(13, 14);
    cmap.one_link(14, 15);
    cmap.one_link(15, 16);
    cmap.one_link(16, 11);
    // assemble
    cmap.two_link(2, 10);
    cmap.two_link(3, 11);
    cmap.two_link(9, 12);

    // insert vertices
    cmap.insert_vertex(1, (0.0, 0.0));
    cmap.insert_vertex(2, (1.0, 0.0));
    cmap.insert_vertex(6, (2.0, 0.0));
    cmap.insert_vertex(4, (0.0, 1.0));
    cmap.insert_vertex(3, (1.0, 1.0));
    cmap.insert_vertex(7, (2.0, 1.0));
    cmap.insert_vertex(16, (0.0, 2.0));
    cmap.insert_vertex(15, (1.0, 3.0));
    cmap.insert_vertex(14, (2.0, 2.0));

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
