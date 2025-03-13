// split_edge

use honeycomb_core::cmap::{CMap2, CMapBuilder, NULL_DART_ID};
use honeycomb_core::geometry::{CoordsFloat, Vertex2};

use super::*;

fn newmap<T: CoordsFloat>(n: usize) -> CMap2<T> {
    CMapBuilder::from_n_darts(n).build().unwrap()
}

mod vertices {
    use honeycomb_core::stm::atomically_with_err;

    use super::*;

    #[test]
    fn split_edge_complete() {
        // before
        //    <--6---   <--5---   <--4---
        //  1         2         3         4
        //    ---1-->   ---2-->   ---3-->
        let mut map: CMap2<f64> = newmap(6);
        map.force_link::<1>(1, 2).unwrap();
        map.force_link::<1>(2, 3).unwrap();
        map.force_link::<1>(4, 5).unwrap();
        map.force_link::<1>(5, 6).unwrap();
        map.force_link::<2>(1, 6).unwrap();
        map.force_link::<2>(2, 5).unwrap();
        map.force_link::<2>(3, 4).unwrap();
        map.force_write_vertex(1, (0.0, 0.0));
        map.force_write_vertex(2, (1.0, 0.0));
        map.force_write_vertex(3, (2.0, 0.0));
        map.force_write_vertex(4, (3.0, 0.0));
        // split
        let nds = map.add_free_darts(2);
        let res = atomically_with_err(|trans| {
            insert_vertex_in_edge(&map, trans, 2, (nds, nds + 1), None)
        });
        assert!(res.is_ok());
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

        assert_eq!(map.force_read_vertex(2), Some(Vertex2::from((1.0, 0.0))));
        assert_eq!(map.force_read_vertex(7), Some(Vertex2::from((1.5, 0.0))));
        assert_eq!(map.force_read_vertex(3), Some(Vertex2::from((2.0, 0.0))));
    }

    #[test]
    fn split_edge_isolated() {
        // before
        //    <--2---
        //  1         2
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.force_link::<2>(1, 2).unwrap();
        map.force_write_vertex(1, (0.0, 0.0));
        map.force_write_vertex(2, (1.0, 0.0));
        // split
        let nds = map.add_free_darts(2);
        let res = atomically_with_err(|trans| {
            insert_vertex_in_edge(&map, trans, 1, (nds, nds + 1), Some(0.6))
        });
        assert!(res.is_ok());
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

        assert_eq!(map.force_read_vertex(1), Some(Vertex2::from((0.0, 0.0))));
        assert_eq!(map.force_read_vertex(3), Some(Vertex2::from((0.6, 0.0))));
        assert_eq!(map.force_read_vertex(2), Some(Vertex2::from((1.0, 0.0))));
    }

    #[test]
    fn split_single_dart() {
        // before
        //  1 -----> 2 ->
        let mut map: CMap2<f64> = newmap(2);
        map.force_link::<1>(1, 2).unwrap();
        map.force_write_vertex(1, (0.0, 0.0));
        map.force_write_vertex(2, (1.0, 0.0));
        // split
        let nd = map.add_free_dart(); // a single dart is enough in this case
        let res = atomically_with_err(|trans| {
            insert_vertex_in_edge(&map, trans, 1, (nd, NULL_DART_ID), None)
        });
        assert!(res.is_ok());
        // after
        //  1 -> 3 -> 2 ->
        assert_eq!(map.beta::<1>(1), 3);
        assert_eq!(map.beta::<1>(3), 2);
        assert_eq!(map.beta::<2>(3), NULL_DART_ID);
        assert_eq!(map.force_read_vertex(3), Some(Vertex2::from((0.5, 0.0))));
    }

    #[test]
    fn split_edge_missing_vertex() {
        //    <--2---
        //  1         ?
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.force_link::<2>(1, 2).unwrap();
        map.force_write_vertex(1, (0.0, 0.0));
        // map.force_write_vertex(2, (1.0, 0.0)); missing vertex!
        // split
        let nds = map.add_free_darts(2);
        let res = atomically_with_err(|trans| {
            insert_vertex_in_edge(&map, trans, 1, (nds, nds + 1), None)
        });
        assert!(res.is_err_and(|e| e == VertexInsertionError::UndefinedEdge));
    }

    // splitn_edge

    #[test]
    fn splitn_edge_complete() {
        // before
        //    <--6---   <--5---   <--4---
        //  1         2         3         4
        //    ---1-->   ---2-->   ---3-->
        let mut map: CMap2<f64> = newmap(6);
        map.force_link::<1>(1, 2).unwrap();
        map.force_link::<1>(2, 3).unwrap();
        map.force_link::<1>(4, 5).unwrap();
        map.force_link::<1>(5, 6).unwrap();
        map.force_link::<2>(1, 6).unwrap();
        map.force_link::<2>(2, 5).unwrap();
        map.force_link::<2>(3, 4).unwrap();
        map.force_write_vertex(1, (0.0, 0.0));
        map.force_write_vertex(2, (1.0, 0.0));
        map.force_write_vertex(3, (2.0, 0.0));
        map.force_write_vertex(4, (3.0, 0.0));
        // split
        let nds = map.add_free_darts(6);
        let new_darts = (nds..nds + 6).collect::<Vec<_>>();
        let res = atomically_with_err(|trans| {
            insert_vertices_in_edge(&map, trans, 2, &new_darts, &[0.25, 0.50, 0.75])
        });
        assert!(res.is_ok());
        // after
        //    <--6---             <--4---
        //  1         2 -7-8-9- 3         4
        //    ---1-->             ---3-->
        assert_eq!(&new_darts[0..3], &[7, 8, 9]);
        assert_eq!(map.force_read_vertex(7), Some(Vertex2(1.25, 0.0)));
        assert_eq!(map.force_read_vertex(8), Some(Vertex2(1.50, 0.0)));
        assert_eq!(map.force_read_vertex(9), Some(Vertex2(1.75, 0.0)));

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
        let mut map: CMap2<f64> = newmap(2);
        map.force_link::<2>(1, 2).unwrap();
        map.force_write_vertex(1, (0.0, 0.0));
        map.force_write_vertex(2, (1.0, 0.0));
        // split
        let nds = map.add_free_darts(6);
        let new_darts = (nds..nds + 6).collect::<Vec<_>>();
        let res = atomically_with_err(|trans| {
            insert_vertices_on_edge(&map, trans, 1, &new_darts, &[0.25, 0.50, 0.75])
        });
        assert!(res.is_ok());
        // after
        //    <-<-<-<
        //  1 -3-4-5- 2
        //    >->->->
        assert_eq!(&new_darts[0..3], &[3, 4, 5]);
        assert_eq!(map.force_read_vertex(3), Some(Vertex2(0.25, 0.0)));
        assert_eq!(map.force_read_vertex(4), Some(Vertex2(0.50, 0.0)));
        assert_eq!(map.force_read_vertex(5), Some(Vertex2(0.75, 0.0)));

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
        let mut map: CMap2<f64> = newmap(2);
        map.force_link::<1>(1, 2).unwrap();
        map.force_write_vertex(1, (0.0, 0.0));
        map.force_write_vertex(2, (1.0, 0.0));
        // split
        let nds = map.add_free_darts(3);
        let res = atomically_with_err(|trans| {
            insert_vertices_in_edge(
                &map,
                trans,
                1,
                &[
                    nds,
                    nds + 1,
                    nds + 2,
                    NULL_DART_ID,
                    NULL_DART_ID,
                    NULL_DART_ID,
                ],
                &[0.25, 0.50, 0.75],
            )
        });
        assert!(res.is_ok());
        // after
        //  1 -> 3 -> 4 -> 5 -> 2 ->
        // assert_eq!(&new_darts, &[3, 4, 5]);
        assert_eq!(map.force_read_vertex(3), Some(Vertex2(0.25, 0.0)));
        assert_eq!(map.force_read_vertex(4), Some(Vertex2(0.50, 0.0)));
        assert_eq!(map.force_read_vertex(5), Some(Vertex2(0.75, 0.0)));

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
    fn splitn_edge_missing_vertex() {
        //    <--2---
        //  1         ?
        //    ---1-->
        let mut map: CMap2<f64> = newmap(2);
        map.force_link::<2>(1, 2).unwrap();
        map.force_write_vertex(1, (0.0, 0.0));
        // map.force_write_vertex(2, (1.0, 0.0)); missing vertex!
        // split
        let nds = map.add_free_darts(6);
        let res = atomically_with_err(|trans| {
            insert_vertices_in_edge(
                &map,
                trans,
                1,
                &[nds, nds + 1, nds + 2, nds + 3, nds + 4, nds + 5],
                &[0.25, 0.50, 0.75],
            )
        });
        assert!(res.is_err_and(|e| e == VertexInsertionError::UndefinedEdge));
    }
}
