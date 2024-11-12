// split_edge

use super::*;
use honeycomb_core::cmap::{CMap2, CMapBuilder, NULL_DART_ID};
use honeycomb_core::geometry::Vertex2;
use honeycomb_core::prelude::CoordsFloat;

fn newmap<T: CoordsFloat>(n: usize) -> CMap2<T> {
    CMapBuilder::default().n_darts(n).build().unwrap()
}

mod standard {
    use honeycomb_core::cmap::{DartId, EdgeId, VertexId};

    use super::*;

    #[test]
    fn split_edge_complete() {
        // before
        //    <--6---   <--5---   <--4---
        //  1         2         3         4
        //    ---1-->   ---2-->   ---3-->
        let mut map: CMap2<f64> = newmap(6);
        map.one_link(DartId(1), DartId(2));
        map.one_link(DartId(2), DartId(3));
        map.one_link(DartId(4), DartId(5));
        map.one_link(DartId(5), DartId(6));
        map.two_link(DartId(1), DartId(6));
        map.two_link(DartId(2), DartId(5));
        map.two_link(DartId(3), DartId(4));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        map.insert_vertex(VertexId(3), (2.0, 0.0));
        map.insert_vertex(VertexId(4), (3.0, 0.0));
        // split
        assert!(split_edge(&mut map, EdgeId(2), None).is_ok());
        // after
        //    <--6---   <8- <5-   <--4---
        //  1         2    7    3         4
        //    ---1-->   -2> -7>   ---3-->
        assert_eq!(map.beta::<2>(DartId(2)), DartId(8));
        assert_eq!(map.beta::<1>(DartId(1)), DartId(2));
        assert_eq!(map.beta::<1>(DartId(2)), DartId(7));
        assert_eq!(map.beta::<1>(DartId(7)), DartId(3));

        assert_eq!(map.beta::<2>(DartId(5)), DartId(7));
        assert_eq!(map.beta::<1>(DartId(4)), DartId(5));
        assert_eq!(map.beta::<1>(DartId(5)), DartId(8));
        assert_eq!(map.beta::<1>(DartId(8)), DartId(6));

        assert_eq!(map.vertex_id(DartId(8)), VertexId(7));
        assert_eq!(map.vertex_id(DartId(7)), VertexId(7));

        assert_eq!(map.vertex(VertexId(2)), Some(Vertex2::from((1.0, 0.0))));
        assert_eq!(map.vertex(VertexId(7)), Some(Vertex2::from((1.5, 0.0))));
        assert_eq!(map.vertex(VertexId(3)), Some(Vertex2::from((2.0, 0.0))));
    }

    #[test]
    fn split_edge_isolated() {
        // before
        //    <--2---
        //  1         2
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.two_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        // split
        assert!(split_edge(&mut map, EdgeId(1), Some(0.6)).is_ok());
        // after
        //    <-4- <2-
        //  1     3    2
        //    -1-> -3>
        assert_eq!(map.beta::<2>(DartId(1)), DartId(4));
        assert_eq!(map.beta::<1>(DartId(1)), DartId(3));

        assert_eq!(map.beta::<2>(DartId(2)), DartId(3));
        assert_eq!(map.beta::<1>(DartId(2)), DartId(4));

        assert_eq!(map.vertex_id(DartId(3)), VertexId(3));
        assert_eq!(map.vertex_id(DartId(4)), VertexId(3));

        assert_eq!(map.vertex(VertexId(1)), Some(Vertex2::from((0.0, 0.0))));
        assert_eq!(map.vertex(VertexId(3)), Some(Vertex2::from((0.6, 0.0))));
        assert_eq!(map.vertex(VertexId(2)), Some(Vertex2::from((1.0, 0.0))));
    }

    #[test]
    fn split_single_dart() {
        // before
        //  1 -----> 2 ->
        let mut map: CMap2<f64> = newmap(2);
        map.one_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        // split
        assert!(split_edge(&mut map, EdgeId(1), None).is_ok());
        // after
        //  1 -> 3 -> 2 ->
        assert_eq!(map.beta::<1>(DartId(1)), DartId(3));
        assert_eq!(map.beta::<1>(DartId(3)), DartId(2));
        assert_eq!(map.beta::<2>(DartId(3)), NULL_DART_ID);
        assert_eq!(map.vertex(VertexId(3)), Some(Vertex2::from((0.5, 0.0))));
    }

    #[test]
    fn split_edge_missing_vertex() {
        //    <--2---
        //  1         ?
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.two_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        // map.insert_vertex(VertexId(2), (1.0, 0.0)); missing vertex!
        // split
        assert!(split_edge(&mut map, EdgeId(1), None)
            .is_err_and(|e| e == SplitEdgeError::UndefinedEdge));
    }

    // splitn_edge

    #[test]
    fn splitn_edge_complete() {
        // before
        //    <--6---   <--5---   <--4---
        //  1         2         3         4
        //    ---1-->   ---2-->   ---3-->
        let mut map: CMap2<f64> = newmap(6);
        map.one_link(DartId(1), DartId(2));
        map.one_link(DartId(2), DartId(3));
        map.one_link(DartId(4), DartId(5));
        map.one_link(DartId(5), DartId(6));
        map.two_link(DartId(1), DartId(6));
        map.two_link(DartId(2), DartId(5));
        map.two_link(DartId(3), DartId(4));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        map.insert_vertex(VertexId(3), (2.0, 0.0));
        map.insert_vertex(VertexId(4), (3.0, 0.0));
        // split
        assert!(splitn_edge(&mut map, EdgeId(2), [0.25, 0.50, 0.75]).is_ok());
        // after
        //    <--6---             <--4---
        //  1         2 -7-8-9- 3         4
        //    ---1-->             ---3-->
        let new_darts = [
            map.beta::<1>(DartId(2)),
            map.beta::<1>(map.beta::<1>(DartId(2))),
            map.beta::<1>(map.beta::<1>(map.beta::<1>(DartId(2)))),
        ];
        assert_eq!(&new_darts, &[DartId(7), DartId(8), DartId(9)]);
        assert_eq!(map.vertex(VertexId(7)), Some(Vertex2(1.25, 0.0)));
        assert_eq!(map.vertex(VertexId(8)), Some(Vertex2(1.50, 0.0)));
        assert_eq!(map.vertex(VertexId(9)), Some(Vertex2(1.75, 0.0)));

        assert_eq!(map.beta::<1>(DartId(2)), DartId(7));
        assert_eq!(map.beta::<1>(DartId(7)), DartId(8));
        assert_eq!(map.beta::<1>(DartId(8)), DartId(9));
        assert_eq!(map.beta::<1>(DartId(9)), DartId(3));

        assert_eq!(map.beta::<1>(DartId(5)), DartId(0));
        assert_eq!(map.beta::<1>(DartId(0)), DartId(1));
        assert_eq!(map.beta::<1>(DartId(1)), DartId(2));
        assert_eq!(map.beta::<1>(DartId(2)), DartId(6));

        assert_eq!(map.beta::<2>(DartId(2)), DartId(2));
        assert_eq!(map.beta::<2>(DartId(7)), DartId(1));
        assert_eq!(map.beta::<2>(DartId(8)), DartId(0));
        assert_eq!(map.beta::<2>(DartId(9)), DartId(5));
    }

    #[test]
    fn splitn_edge_isolated() {
        // before
        //    <--2---
        //  1         2
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.two_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        // split
        assert!(splitn_edge(&mut map, EdgeId(1), [0.25, 0.50, 0.75]).is_ok());
        // after
        //    <-<-<-<
        //  1 -3-4-5- 2
        //    >->->->
        let new_darts = [
            map.beta::<1>(DartId(1)),
            map.beta::<1>(map.beta::<1>(DartId(1))),
            map.beta::<1>(map.beta::<1>(map.beta::<1>(DartId(1)))),
        ];
        assert_eq!(&new_darts, &[DartId(3), DartId(4), DartId(5)]);

        assert_eq!(map.beta::<1>(DartId(1)), DartId(3));
        assert_eq!(map.beta::<1>(DartId(3)), DartId(4));
        assert_eq!(map.beta::<1>(DartId(4)), DartId(5));
        assert_eq!(map.beta::<1>(DartId(5)), NULL_DART_ID);

        assert_eq!(map.beta::<1>(DartId(2)), DartId(6));
        assert_eq!(map.beta::<1>(DartId(6)), DartId(7));
        assert_eq!(map.beta::<1>(DartId(7)), DartId(8));
        assert_eq!(map.beta::<1>(DartId(8)), NULL_DART_ID);

        assert_eq!(map.beta::<2>(DartId(1)), DartId(8));
        assert_eq!(map.beta::<2>(DartId(3)), DartId(7));
        assert_eq!(map.beta::<2>(DartId(4)), DartId(6));
        assert_eq!(map.beta::<2>(DartId(5)), DartId(2));
    }

    #[test]
    fn splitn_single_dart() {
        // before
        //  1 -----> 2 ->
        let mut map: CMap2<f64> = newmap(2);
        map.one_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        // split
        assert!(splitn_edge(&mut map, EdgeId(1), [0.25, 0.50, 0.75]).is_ok());
        let new_darts = [
            map.beta::<1>(DartId(1)),
            map.beta::<1>(map.beta::<1>(DartId(1))),
            map.beta::<1>(map.beta::<1>(map.beta::<1>(DartId(1)))),
        ];
        // after
        //  1 -> 3 -> 4 -> 5 -> 2 ->
        assert_eq!(&new_darts, &[DartId(3), DartId(4), DartId(5)]);
        assert_eq!(map.vertex(VertexId(3)), Some(Vertex2(0.25, 0.0)));
        assert_eq!(map.vertex(VertexId(4)), Some(Vertex2(0.50, 0.0)));
        assert_eq!(map.vertex(VertexId(5)), Some(Vertex2(0.75, 0.0)));

        assert_eq!(map.beta::<1>(DartId(1)), DartId(3));
        assert_eq!(map.beta::<1>(DartId(3)), DartId(4));
        assert_eq!(map.beta::<1>(DartId(4)), DartId(5));
        assert_eq!(map.beta::<1>(DartId(5)), DartId(2));

        assert_eq!(map.beta::<2>(DartId(1)), NULL_DART_ID);
        assert_eq!(map.beta::<2>(DartId(3)), NULL_DART_ID);
        assert_eq!(map.beta::<2>(DartId(4)), NULL_DART_ID);
        assert_eq!(map.beta::<2>(DartId(5)), NULL_DART_ID);
    }

    #[test]
    fn splitn_edge_missing_vertex() {
        //    <--2---
        //  1         ?
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.two_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        // map.insert_vertex(VertexId(2), (1.0, 0.0)); missing vertex!
        // split
        assert!(splitn_edge(&mut map, EdgeId(1), [0.25, 0.50, 0.75])
            .is_err_and(|e| e == SplitEdgeError::UndefinedEdge));
    }
}

mod noalloc {
    use honeycomb_core::cmap::{DartId, EdgeId, VertexId};

    use super::*;

    #[test]
    fn split_edge_complete() {
        // before
        //    <--6---   <--5---   <--4---
        //  1         2         3         4
        //    ---1-->   ---2-->   ---3-->
        let mut map: CMap2<f64> = newmap(6);
        map.one_link(DartId(1), DartId(2));
        map.one_link(DartId(2), DartId(3));
        map.one_link(DartId(4), DartId(5));
        map.one_link(DartId(5), DartId(6));
        map.two_link(DartId(1), DartId(6));
        map.two_link(DartId(2), DartId(5));
        map.two_link(DartId(3), DartId(4));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        map.insert_vertex(VertexId(3), (2.0, 0.0));
        map.insert_vertex(VertexId(4), (3.0, 0.0));
        // split
        let nds = map.add_free_darts(2);
        assert!(split_edge_noalloc(&mut map, EdgeId(2), (nds, DartId(nds.0 + 1)), None).is_ok());
        // after
        //    <--6---   <8- <5-   <--4---
        //  1         2    7    3         4
        //    ---1-->   -2> -7>   ---3-->
        assert_eq!(map.beta::<2>(DartId(2)), DartId(8));
        assert_eq!(map.beta::<1>(DartId(1)), DartId(2));
        assert_eq!(map.beta::<1>(DartId(2)), DartId(7));
        assert_eq!(map.beta::<1>(DartId(7)), DartId(3));

        assert_eq!(map.beta::<2>(DartId(5)), DartId(7));
        assert_eq!(map.beta::<1>(DartId(4)), DartId(5));
        assert_eq!(map.beta::<1>(DartId(5)), DartId(8));
        assert_eq!(map.beta::<1>(DartId(8)), DartId(6));

        assert_eq!(map.vertex_id(DartId(8)), VertexId(7));
        assert_eq!(map.vertex_id(DartId(7)), VertexId(7));

        assert_eq!(map.vertex(VertexId(2)), Some(Vertex2::from((1.0, 0.0))));
        assert_eq!(map.vertex(VertexId(7)), Some(Vertex2::from((1.5, 0.0))));
        assert_eq!(map.vertex(VertexId(3)), Some(Vertex2::from((2.0, 0.0))));
    }

    #[test]
    fn split_edge_isolated() {
        // before
        //    <--2---
        //  1         2
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.two_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        // split
        let nds = map.add_free_darts(2);
        assert!(
            split_edge_noalloc(&mut map, EdgeId(1), (nds, DartId(nds.0 + 1)), Some(0.6)).is_ok()
        );
        // after
        //    <-4- <2-
        //  1     3    2
        //    -1-> -3>
        assert_eq!(map.beta::<2>(DartId(1)), DartId(4));
        assert_eq!(map.beta::<1>(DartId(1)), DartId(3));

        assert_eq!(map.beta::<2>(DartId(2)), DartId(3));
        assert_eq!(map.beta::<1>(DartId(2)), DartId(4));

        assert_eq!(map.vertex_id(DartId(3)), VertexId(3));
        assert_eq!(map.vertex_id(DartId(4)), VertexId(3));

        assert_eq!(map.vertex(VertexId(1)), Some(Vertex2::from((0.0, 0.0))));
        assert_eq!(map.vertex(VertexId(3)), Some(Vertex2::from((0.6, 0.0))));
        assert_eq!(map.vertex(VertexId(2)), Some(Vertex2::from((1.0, 0.0))));
    }

    #[test]
    fn split_single_dart() {
        // before
        //  1 -----> 2 ->
        let mut map: CMap2<f64> = newmap(2);
        map.one_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        // split
        let nd = map.add_free_dart(); // a single dart is enough in this case
        assert!(split_edge_noalloc(&mut map, EdgeId(1), (nd, NULL_DART_ID), None).is_ok());
        // after
        //  1 -> 3 -> 2 ->
        assert_eq!(map.beta::<1>(DartId(1)), DartId(3));
        assert_eq!(map.beta::<1>(DartId(3)), DartId(2));
        assert_eq!(map.beta::<2>(DartId(3)), NULL_DART_ID);
        assert_eq!(map.vertex(VertexId(3)), Some(Vertex2::from((0.5, 0.0))));
    }

    #[test]
    fn split_edge_missing_vertex() {
        //    <--2---
        //  1         ?
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.two_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        // map.insert_vertex(VertexId(2), (1.0, 0.0)); missing vertex!
        // split
        let nds = map.add_free_darts(2);
        assert!(
            split_edge_noalloc(&mut map, EdgeId(1), (nds, DartId(nds.0 + 1)), None)
                .is_err_and(|e| e == SplitEdgeError::UndefinedEdge)
        );
    }

    // splitn_edge

    #[test]
    fn splitn_edge_complete() {
        // before
        //    <--6---   <--5---   <--4---
        //  1         2         3         4
        //    ---1-->   ---2-->   ---3-->
        let mut map: CMap2<f64> = newmap(6);
        map.one_link(DartId(1), DartId(2));
        map.one_link(DartId(2), DartId(3));
        map.one_link(DartId(4), DartId(5));
        map.one_link(DartId(5), DartId(6));
        map.two_link(DartId(1), DartId(6));
        map.two_link(DartId(2), DartId(5));
        map.two_link(DartId(3), DartId(4));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        map.insert_vertex(VertexId(3), (2.0, 0.0));
        map.insert_vertex(VertexId(4), (3.0, 0.0));
        // split
        let nds = map.add_free_darts(6).0;
        let new_darts = (nds..nds + 6).map(DartId).collect::<Vec<_>>();
        assert!(splitn_edge_no_alloc(&mut map, EdgeId(2), &new_darts, &[0.25, 0.50, 0.75]).is_ok());
        // after
        //    <--6---             <--4---
        //  1         2 -7-8-9- 3         4
        //    ---1-->             ---3-->
        assert_eq!(&new_darts[0..3], &[DartId(7), DartId(8), DartId(9)]);
        assert_eq!(map.vertex(VertexId(7)), Some(Vertex2(1.25, 0.0)));
        assert_eq!(map.vertex(VertexId(8)), Some(Vertex2(1.50, 0.0)));
        assert_eq!(map.vertex(VertexId(9)), Some(Vertex2(1.75, 0.0)));

        assert_eq!(map.beta::<1>(DartId(2)), DartId(7));
        assert_eq!(map.beta::<1>(DartId(7)), DartId(8));
        assert_eq!(map.beta::<1>(DartId(8)), DartId(9));
        assert_eq!(map.beta::<1>(DartId(9)), DartId(3));

        assert_eq!(map.beta::<1>(DartId(5)), DartId(10));
        assert_eq!(map.beta::<1>(DartId(10)), DartId(11));
        assert_eq!(map.beta::<1>(DartId(11)), DartId(12));
        assert_eq!(map.beta::<1>(DartId(12)), DartId(6));

        assert_eq!(map.beta::<2>(DartId(2)), DartId(12));
        assert_eq!(map.beta::<2>(DartId(7)), DartId(11));
        assert_eq!(map.beta::<2>(DartId(8)), DartId(10));
        assert_eq!(map.beta::<2>(DartId(9)), DartId(5));
    }

    #[test]
    fn splitn_edge_isolated() {
        // before
        //    <--2---
        //  1         2
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.two_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        // split
        let nds = map.add_free_darts(6).0;
        let new_darts = (nds..nds + 6).map(DartId).collect::<Vec<_>>();
        assert!(splitn_edge_no_alloc(&mut map, EdgeId(1), &new_darts, &[0.25, 0.50, 0.75]).is_ok());
        // after
        //    <-<-<-<
        //  1 -3-4-5- 2
        //    >->->->
        assert_eq!(&new_darts[0..3], &[DartId(3), DartId(4), DartId(5)]);
        assert_eq!(map.vertex(VertexId(3)), Some(Vertex2(0.25, 0.0)));
        assert_eq!(map.vertex(VertexId(4)), Some(Vertex2(0.50, 0.0)));
        assert_eq!(map.vertex(VertexId(5)), Some(Vertex2(0.75, 0.0)));

        assert_eq!(map.beta::<1>(DartId(1)), DartId(3));
        assert_eq!(map.beta::<1>(DartId(3)), DartId(4));
        assert_eq!(map.beta::<1>(DartId(4)), DartId(5));
        assert_eq!(map.beta::<1>(DartId(5)), NULL_DART_ID);

        assert_eq!(map.beta::<1>(DartId(2)), DartId(6));
        assert_eq!(map.beta::<1>(DartId(6)), DartId(7));
        assert_eq!(map.beta::<1>(DartId(7)), DartId(8));
        assert_eq!(map.beta::<1>(DartId(8)), NULL_DART_ID);

        assert_eq!(map.beta::<2>(DartId(1)), DartId(8));
        assert_eq!(map.beta::<2>(DartId(3)), DartId(7));
        assert_eq!(map.beta::<2>(DartId(4)), DartId(6));
        assert_eq!(map.beta::<2>(DartId(5)), DartId(2));
    }

    #[test]
    fn splitn_single_dart() {
        // before
        //  1 -----> 2 ->
        let mut map: CMap2<f64> = newmap(2);
        map.one_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        map.insert_vertex(VertexId(2), (1.0, 0.0));
        // split
        let nds = map.add_free_darts(3);
        assert!(splitn_edge_no_alloc(
            &mut map,
            EdgeId(1),
            &[
                nds,
                DartId(nds.0 + 1),
                DartId(nds.0 + 2),
                NULL_DART_ID,
                NULL_DART_ID,
                NULL_DART_ID,
            ],
            &[0.25, 0.50, 0.75],
        )
        .is_ok());
        // after
        //  1 -> 3 -> 4 -> 5 -> 2 ->
        // assert_eq!(&new_darts, &[3, 4, 5]);
        assert_eq!(map.vertex(VertexId(3)), Some(Vertex2(0.25, 0.0)));
        assert_eq!(map.vertex(VertexId(4)), Some(Vertex2(0.50, 0.0)));
        assert_eq!(map.vertex(VertexId(5)), Some(Vertex2(0.75, 0.0)));

        assert_eq!(map.beta::<1>(DartId(1)), DartId(3));
        assert_eq!(map.beta::<1>(DartId(3)), DartId(4));
        assert_eq!(map.beta::<1>(DartId(4)), DartId(5));
        assert_eq!(map.beta::<1>(DartId(5)), DartId(2));

        assert_eq!(map.beta::<2>(DartId(1)), NULL_DART_ID);
        assert_eq!(map.beta::<2>(DartId(3)), NULL_DART_ID);
        assert_eq!(map.beta::<2>(DartId(4)), NULL_DART_ID);
        assert_eq!(map.beta::<2>(DartId(5)), NULL_DART_ID);
    }

    #[test]
    fn splitn_edge_missing_vertex() {
        //    <--2---
        //  1         ?
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.two_link(DartId(1), DartId(2));
        map.insert_vertex(VertexId(1), (0.0, 0.0));
        // map.insert_vertex(DartId(2), (1.0, 0.0)); missing vertex!
        // split
        let nds = map.add_free_darts(6).0;
        assert!(splitn_edge_no_alloc(
            &mut map,
            EdgeId(1),
            &[
                DartId(nds),
                DartId(nds + 1),
                DartId(nds + 2),
                DartId(nds + 3),
                DartId(nds + 4),
                DartId(nds + 5)
            ],
            &[0.25, 0.50, 0.75],
        )
        .is_err_and(|e| e == SplitEdgeError::UndefinedEdge));
    }
}
