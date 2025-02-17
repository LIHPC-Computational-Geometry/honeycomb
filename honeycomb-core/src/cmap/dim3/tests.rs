use crate::{
    attributes::{AttrSparseVec, AttributeBind, AttributeError, AttributeUpdate},
    cmap::{CMap3, DartIdType, Orbit3, OrbitPolicy, SewError, VertexIdType},
    geometry::Vertex3,
    stm::{atomically, atomically_with_err, StmError, TVar, TransactionError},
};

#[test]
fn example_test() {
    // Build a tetrahedron (A)
    let mut map: CMap3<f64> = CMap3::new(12); // 3*4 darts

    // face z- (base)
    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(2, 3).unwrap();
    map.force_link::<1>(3, 1).unwrap();
    // face y-
    map.force_link::<1>(4, 5).unwrap();
    map.force_link::<1>(5, 6).unwrap();
    map.force_link::<1>(6, 4).unwrap();
    // face x-
    map.force_link::<1>(7, 8).unwrap();
    map.force_link::<1>(8, 9).unwrap();
    map.force_link::<1>(9, 7).unwrap();
    // face x+/y+
    map.force_link::<1>(10, 11).unwrap();
    map.force_link::<1>(11, 12).unwrap();
    map.force_link::<1>(12, 10).unwrap();
    // link triangles to get the tet
    map.force_link::<2>(1, 4).unwrap();
    map.force_link::<2>(2, 7).unwrap();
    map.force_link::<2>(3, 10).unwrap();
    map.force_link::<2>(5, 12).unwrap();
    map.force_link::<2>(6, 8).unwrap();
    map.force_link::<2>(9, 11).unwrap();

    // putting this in a scope to force dropping the iterator before the next mutable borrow
    {
        let mut vertices = map.iter_vertices();
        assert_eq!(vertices.next(), Some(1));
        assert_eq!(vertices.next(), Some(2));
        assert_eq!(vertices.next(), Some(3));
        assert_eq!(vertices.next(), Some(6));
        assert_eq!(vertices.next(), None);
    }

    map.force_write_vertex(1, (1.0, 0.0, 0.0));
    map.force_write_vertex(2, (0.0, 0.0, 0.0));
    map.force_write_vertex(3, (0.0, 0.5, 0.0));
    map.force_write_vertex(6, (0.5, 0.25, 1.0));

    // Build a second tetrahedron (B)

    let _ = map.add_free_darts(12);
    // face z- (base)
    map.force_link::<1>(13, 14).unwrap();
    map.force_link::<1>(14, 15).unwrap();
    map.force_link::<1>(15, 13).unwrap();
    // face x-/y-
    map.force_link::<1>(16, 17).unwrap();
    map.force_link::<1>(17, 18).unwrap();
    map.force_link::<1>(18, 16).unwrap();
    // face y+
    map.force_link::<1>(19, 20).unwrap();
    map.force_link::<1>(20, 21).unwrap();
    map.force_link::<1>(21, 19).unwrap();
    // face x+
    map.force_link::<1>(22, 23).unwrap();
    map.force_link::<1>(23, 24).unwrap();
    map.force_link::<1>(24, 22).unwrap();
    // link triangles to get the tet
    map.force_link::<2>(13, 16).unwrap();
    map.force_link::<2>(14, 19).unwrap();
    map.force_link::<2>(15, 22).unwrap();
    map.force_link::<2>(17, 24).unwrap();
    map.force_link::<2>(18, 20).unwrap();
    map.force_link::<2>(21, 23).unwrap();

    map.force_write_vertex(13, (2.5, 1.5, 0.0));
    map.force_write_vertex(14, (1.5, 2.0, 0.0));
    map.force_write_vertex(15, (2.5, 2.0, 0.0));
    map.force_write_vertex(18, (1.5, 1.75, 1.0));

    {
        let mut volumes = map.iter_volumes();
        assert_eq!(volumes.next(), Some(1));
        assert_eq!(volumes.next(), Some(13));
        assert_eq!(volumes.next(), None);
        let mut faces = map.iter_faces();
        assert_eq!(faces.next(), Some(1));
        assert_eq!(faces.next(), Some(4));
        assert_eq!(faces.next(), Some(7));
        assert_eq!(faces.next(), Some(10));
        assert_eq!(faces.next(), Some(13));
        assert_eq!(faces.next(), Some(16));
        assert_eq!(faces.next(), Some(19));
        assert_eq!(faces.next(), Some(22));
        assert_eq!(faces.next(), None);
    }

    // Sew both tetrahedrons along a face (C)

    println!("v d10: {:?}", map.force_read_vertex(map.vertex_id(10)));
    println!(
        "v b1d10: {:?}",
        map.force_read_vertex(map.vertex_id(map.beta::<1>(10)))
    );
    println!("v d16: {:?}", map.force_read_vertex(map.vertex_id(16)));
    println!(
        "v b1d16: {:?}",
        map.force_read_vertex(map.vertex_id(map.beta::<1>(16)))
    );

    assert_eq!(map.n_vertices(), 8);
    map.force_sew::<3>(10, 16).unwrap();
    assert_eq!(map.n_vertices(), 5);

    // this results in a quad-base pyramid
    // the pyramid is split in two volumes along the (base) diagonal plane
    {
        let mut faces = map.iter_faces();
        assert_eq!(faces.next(), Some(1));
        assert_eq!(faces.next(), Some(4));
        assert_eq!(faces.next(), Some(7));
        assert_eq!(faces.next(), Some(10));
        assert_eq!(faces.next(), Some(13));
        // assert_eq!(faces.next(), Some(16)); // now fused with 10
        assert_eq!(faces.next(), Some(19));
        assert_eq!(faces.next(), Some(22));
        assert_eq!(faces.next(), None);
        // there should be 9 edges total; quad base pyramid (8) + the base split diagonal (1)
        assert_eq!(map.iter_edges().count(), 9);
    }

    // Adjust shared vertices (D)

    // this makes it a symetrical square-base pyramid
    assert_eq!(
        map.force_write_vertex(3, (0.0, 1.0, 0.0)),
        Some(Vertex3(0.75, 1.25, 0.0))
    );
    assert_eq!(
        map.force_write_vertex(1, (1.0, 0.0, 0.0)),
        Some(Vertex3(1.75, 0.75, 0.0))
    );
    assert_eq!(
        map.force_write_vertex(6, (0.5, 0.5, 1.0)),
        Some(Vertex3(1.0, 1.0, 1.0))
    );
    assert_eq!(
        map.force_write_vertex(15, (1.0, 1.0, 0.0)),
        Some(Vertex3(2.5, 2.0, 0.0))
    );

    // Remove the split to have a single volume pyramid (E)

    fn rebuild_edge(map: &CMap3<f64>, dart: DartIdType) {
        let b3d = map.beta::<3>(dart);
        let ld = map.beta::<2>(dart);
        let rd = map.beta::<2>(b3d);

        map.force_unsew::<2>(dart).unwrap();
        map.force_unsew::<2>(b3d).unwrap();
        map.force_sew::<2>(ld, rd).unwrap();
    }
    rebuild_edge(&map, 10);
    rebuild_edge(&map, 11);
    rebuild_edge(&map, 12);

    // delete old face components
    map.force_unlink::<1>(10).unwrap();
    map.force_unlink::<1>(11).unwrap();
    map.force_unlink::<1>(12).unwrap();
    map.force_unlink::<3>(10).unwrap();
    map.force_unlink::<3>(11).unwrap();
    map.force_unlink::<3>(12).unwrap();
    map.remove_free_dart(10);
    map.remove_free_dart(11);
    map.remove_free_dart(12);
    map.remove_free_dart(16);
    map.remove_free_dart(17);
    map.remove_free_dart(18);

    {
        let mut volumes = map.iter_volumes();
        assert_eq!(volumes.next(), Some(1));
        assert_eq!(volumes.next(), None);
        let mut faces = map.iter_faces();
        assert_eq!(faces.next(), Some(1)); // base
        assert_eq!(faces.next(), Some(4)); // y-
        assert_eq!(faces.next(), Some(7)); // x-
        assert_eq!(faces.next(), Some(13)); // base
        assert_eq!(faces.next(), Some(19)); // y+
        assert_eq!(faces.next(), Some(22)); // x+
        assert_eq!(faces.next(), None);
    }
}

#[test]
fn example_test_transactional() {
    // Build a tetrahedron (A)
    let mut map: CMap3<f64> = CMap3::new(12); // 3*4 darts

    // face z- (base)
    let res = atomically_with_err(|trans| {
        map.link::<1>(trans, 1, 2)?;
        map.link::<1>(trans, 2, 3)?;
        map.link::<1>(trans, 3, 1)?;
        // face y-
        map.link::<1>(trans, 4, 5)?;
        map.link::<1>(trans, 5, 6)?;
        map.link::<1>(trans, 6, 4)?;
        // face x-
        map.link::<1>(trans, 7, 8)?;
        map.link::<1>(trans, 8, 9)?;
        map.link::<1>(trans, 9, 7)?;
        // face x+/y+
        map.link::<1>(trans, 10, 11)?;
        map.link::<1>(trans, 11, 12)?;
        map.link::<1>(trans, 12, 10)?;
        // link triangles to get the tet
        map.link::<2>(trans, 1, 4)?;
        map.link::<2>(trans, 2, 7)?;
        map.link::<2>(trans, 3, 10)?;
        map.link::<2>(trans, 5, 12)?;
        map.link::<2>(trans, 6, 8)?;
        map.link::<2>(trans, 9, 11)?;
        Ok(())
    });
    assert!(res.is_ok());

    // putting this in a scope to force dropping the iterator before the next mutable borrow
    {
        let mut vertices = map.iter_vertices();
        assert_eq!(vertices.next(), Some(1));
        assert_eq!(vertices.next(), Some(2));
        assert_eq!(vertices.next(), Some(3));
        assert_eq!(vertices.next(), Some(6));
        assert_eq!(vertices.next(), None);
    }

    atomically(|trans| {
        map.write_vertex(trans, 1, (1.0, 0.0, 0.0))?;
        map.write_vertex(trans, 2, (0.0, 0.0, 0.0))?;
        map.write_vertex(trans, 3, (0.0, 0.5, 0.0))?;
        map.write_vertex(trans, 6, (0.5, 0.25, 1.0))?;
        Ok(())
    });

    // Build a second tetrahedron (B)
    let _ = map.add_free_darts(12);
    let res = atomically_with_err(|trans| {
        // face z- (base)
        map.link::<1>(trans, 13, 14)?;
        map.link::<1>(trans, 14, 15)?;
        map.link::<1>(trans, 15, 13)?;
        // face x-/y-
        map.link::<1>(trans, 16, 17)?;
        map.link::<1>(trans, 17, 18)?;
        map.link::<1>(trans, 18, 16)?;
        // face y+
        map.link::<1>(trans, 19, 20)?;
        map.link::<1>(trans, 20, 21)?;
        map.link::<1>(trans, 21, 19)?;
        // face x+
        map.link::<1>(trans, 22, 23)?;
        map.link::<1>(trans, 23, 24)?;
        map.link::<1>(trans, 24, 22)?;
        // link triangles to get the tet
        map.link::<2>(trans, 13, 16)?;
        map.link::<2>(trans, 14, 19)?;
        map.link::<2>(trans, 15, 22)?;
        map.link::<2>(trans, 17, 24)?;
        map.link::<2>(trans, 18, 20)?;
        map.link::<2>(trans, 21, 23)?;

        map.write_vertex(trans, 13, (2.5, 1.5, 0.0))?;
        map.write_vertex(trans, 14, (1.5, 2.0, 0.0))?;
        map.write_vertex(trans, 15, (2.5, 2.0, 0.0))?;
        map.write_vertex(trans, 18, (1.5, 1.75, 1.0))?;
        Ok(())
    });
    assert!(res.is_ok());

    {
        let mut volumes = map.iter_volumes();
        assert_eq!(volumes.next(), Some(1));
        assert_eq!(volumes.next(), Some(13));
        assert_eq!(volumes.next(), None);
        let mut faces = map.iter_faces();
        assert_eq!(faces.next(), Some(1));
        assert_eq!(faces.next(), Some(4));
        assert_eq!(faces.next(), Some(7));
        assert_eq!(faces.next(), Some(10));
        assert_eq!(faces.next(), Some(13));
        assert_eq!(faces.next(), Some(16));
        assert_eq!(faces.next(), Some(19));
        assert_eq!(faces.next(), Some(22));
        assert_eq!(faces.next(), None);
    }

    // Sew both tetrahedrons along a face (C)
    assert_eq!(map.n_vertices(), 8);
    atomically(|trans| {
        assert!(map.sew::<3>(trans, 10, 16).is_ok());
        Ok(())
    });
    assert_eq!(map.n_vertices(), 5);

    // this results in a quad-base pyramid
    // the pyramid is split in two volumes along the (base) diagonal plane
    {
        let mut faces = map.iter_faces();
        assert_eq!(faces.next(), Some(1));
        assert_eq!(faces.next(), Some(4));
        assert_eq!(faces.next(), Some(7));
        assert_eq!(faces.next(), Some(10));
        assert_eq!(faces.next(), Some(13));
        // assert_eq!(faces.next(), Some(16)); // now fused with 10
        assert_eq!(faces.next(), Some(19));
        assert_eq!(faces.next(), Some(22));
        assert_eq!(faces.next(), None);
        // there should be 9 edges total; quad base pyramid (8) + the base split diagonal (1)
        assert_eq!(map.iter_edges().count(), 9);
    }

    // Adjust shared vertices (D)
    atomically(|trans| {
        // this makes it a symetrical square-base pyramid
        assert_eq!(
            map.write_vertex(trans, 3, (0.0, 1.0, 0.0))?,
            Some(Vertex3(0.75, 1.25, 0.0))
        );
        assert_eq!(
            map.write_vertex(trans, 1, (1.0, 0.0, 0.0))?,
            Some(Vertex3(1.75, 0.75, 0.0))
        );
        assert_eq!(
            map.write_vertex(trans, 6, (0.5, 0.5, 1.0))?,
            Some(Vertex3(1.0, 1.0, 1.0))
        );
        assert_eq!(
            map.write_vertex(trans, 15, (1.0, 1.0, 0.0))?,
            Some(Vertex3(2.5, 2.0, 0.0))
        );
        Ok(())
    });

    // Remove the split to have a single volume pyramid (E)

    fn rebuild_edge(map: &CMap3<f64>, dart: DartIdType) {
        atomically(|trans| {
            let b3d = map.beta_transac::<3>(trans, dart)?;
            let ld = map.beta_transac::<2>(trans, dart)?;
            let rd = map.beta_transac::<2>(trans, b3d)?;

            assert!(map.unsew::<2>(trans, dart).is_ok());
            assert!(map.unsew::<2>(trans, b3d).is_ok());
            assert!(map.sew::<2>(trans, ld, rd).is_ok());
            Ok(())
        })
    }
    rebuild_edge(&map, 10);
    rebuild_edge(&map, 11);
    rebuild_edge(&map, 12);

    // delete old face components
    atomically(|trans| {
        assert!(map.unlink::<1>(trans, 10).is_ok());
        assert!(map.unlink::<1>(trans, 11).is_ok());
        assert!(map.unlink::<1>(trans, 12).is_ok());
        assert!(map.unlink::<3>(trans, 10).is_ok());
        assert!(map.unlink::<3>(trans, 11).is_ok());
        assert!(map.unlink::<3>(trans, 12).is_ok());
        Ok(())
    });

    map.remove_free_dart(10);
    map.remove_free_dart(11);
    map.remove_free_dart(12);
    map.remove_free_dart(16);
    map.remove_free_dart(17);
    map.remove_free_dart(18);

    {
        let mut volumes = map.iter_volumes();
        assert_eq!(volumes.next(), Some(1));
        assert_eq!(volumes.next(), None);
        let mut faces = map.iter_faces();
        assert_eq!(faces.next(), Some(1)); // base
        assert_eq!(faces.next(), Some(4)); // y-
        assert_eq!(faces.next(), Some(7)); // x-
        assert_eq!(faces.next(), Some(13)); // base
        assert_eq!(faces.next(), Some(19)); // y+
        assert_eq!(faces.next(), Some(22)); // x+
        assert_eq!(faces.next(), None);
    }
}

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
fn one_sew() {
    let map: CMap3<f64> = CMap3::new(8);
    // map.force_link::<1>(1, 2);
    map.force_link::<1>(2, 3).unwrap();
    map.force_link::<1>(3, 4).unwrap();
    map.force_link::<1>(4, 1).unwrap();
    map.force_write_vertex(1, Vertex3(0.0, 0.0, 0.0));
    map.force_write_vertex(2, Vertex3(1.0, 0.0, 0.0));
    map.force_write_vertex(3, Vertex3(1.0, 1.0, 0.0));
    map.force_write_vertex(4, Vertex3(0.0, 1.0, 0.0));

    map.force_link::<1>(5, 6).unwrap();
    map.force_link::<1>(6, 7).unwrap();
    map.force_link::<1>(7, 8).unwrap();
    // map.force_link::<1>(8, 5);
    map.force_write_vertex(5, Vertex3(0.5, 0.0, 1.0));
    map.force_write_vertex(6, Vertex3(0.0, 0.0, 1.0));
    map.force_write_vertex(7, Vertex3(0.0, 1.0, 1.0));
    map.force_write_vertex(8, Vertex3(1.0, 1.0, 1.0));

    map.force_sew::<3>(1, 5).unwrap();
    assert_eq!(map.beta::<3>(1), 5);
    assert_eq!(map.beta::<3>(2), 8);
    assert_eq!(map.beta::<3>(3), 7);
    assert_eq!(map.beta::<3>(4), 6);

    map.force_sew::<1>(1, 2).unwrap();

    assert_eq!(map.beta::<1>(1), 2);
    assert_eq!(map.beta::<1>(8), 5);
    assert_eq!(map.vertex_id(5), 2);
    assert_eq!(map.force_read_vertex(2), Some(Vertex3(0.75, 0.0, 0.5)));
}

#[test]
fn three_sew() {
    let map: CMap3<f64> = CMap3::new(8);
    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(2, 3).unwrap();
    map.force_link::<1>(3, 4).unwrap();
    map.force_link::<1>(4, 1).unwrap();
    map.force_write_vertex(1, Vertex3(0.0, 0.0, 0.0));
    map.force_write_vertex(2, Vertex3(1.0, 0.0, 0.0));
    map.force_write_vertex(3, Vertex3(1.0, 1.0, 0.0));
    map.force_write_vertex(4, Vertex3(0.0, 1.0, 0.0));

    map.force_link::<1>(5, 6).unwrap();
    map.force_link::<1>(6, 7).unwrap();
    map.force_link::<1>(7, 8).unwrap();
    map.force_link::<1>(8, 5).unwrap();
    map.force_write_vertex(5, Vertex3(0.0, 0.0, 1.0));
    map.force_write_vertex(6, Vertex3(0.0, 1.0, 1.0));
    map.force_write_vertex(7, Vertex3(1.0, 1.0, 1.0));
    map.force_write_vertex(8, Vertex3(1.0, 0.0, 1.0));

    map.force_sew::<3>(1, 8).unwrap();
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex3(0.0, 0.0, 0.5));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex3(1.0, 0.0, 0.5));
    assert_eq!(map.force_read_vertex(3).unwrap(), Vertex3(1.0, 1.0, 0.5));
    assert_eq!(map.force_read_vertex(4).unwrap(), Vertex3(0.0, 1.0, 0.5));
}

#[test]
fn two_sew_bad_orientation() {
    let map: CMap3<f64> = CMap3::new(8);
    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(2, 3).unwrap();
    map.force_link::<1>(3, 4).unwrap();
    map.force_link::<1>(4, 1).unwrap();
    map.force_link::<1>(5, 6).unwrap();
    map.force_link::<1>(6, 7).unwrap();
    map.force_link::<1>(7, 8).unwrap();
    map.force_link::<1>(8, 5).unwrap();
    map.force_write_vertex(1, Vertex3(0.0, 0.0, 0.0));
    map.force_write_vertex(2, Vertex3(1.0, 0.0, 0.0));
    map.force_write_vertex(3, Vertex3(1.0, 1.0, 0.0));
    map.force_write_vertex(4, Vertex3(0.0, 1.0, 0.0));
    map.force_write_vertex(5, Vertex3(0.0, 0.0, 1.0));
    map.force_write_vertex(6, Vertex3(1.0, 0.0, 1.0));
    map.force_write_vertex(7, Vertex3(1.0, 1.0, 1.0));
    map.force_write_vertex(8, Vertex3(0.0, 1.0, 1.0));
    assert!(map
        .force_sew::<2>(1, 5)
        .is_err_and(|e| e == SewError::BadGeometry(2, 1, 5)));
}

#[test]
fn three_sew_bad_orientation() {
    let map: CMap3<f64> = CMap3::new(8);
    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(2, 3).unwrap();
    map.force_link::<1>(3, 4).unwrap();
    map.force_link::<1>(4, 1).unwrap();
    map.force_link::<1>(5, 6).unwrap();
    map.force_link::<1>(6, 7).unwrap();
    map.force_link::<1>(7, 8).unwrap();
    map.force_link::<1>(8, 5).unwrap();
    map.force_write_vertex(1, Vertex3(0.0, 0.0, 0.0));
    map.force_write_vertex(2, Vertex3(1.0, 0.0, 0.0));
    map.force_write_vertex(3, Vertex3(1.0, 1.0, 0.0));
    map.force_write_vertex(4, Vertex3(0.0, 1.0, 0.0));
    map.force_write_vertex(5, Vertex3(0.0, 0.0, 1.0));
    map.force_write_vertex(6, Vertex3(1.0, 0.0, 1.0));
    map.force_write_vertex(7, Vertex3(1.0, 1.0, 1.0));
    map.force_write_vertex(8, Vertex3(0.0, 1.0, 1.0));
    assert!(map
        .force_sew::<3>(1, 5)
        .is_err_and(|e| e == SewError::BadGeometry(3, 1, 5)));
}

// --- PARALLEL

#[test]
fn sew_ordering() {
    loom::model(|| {
        // setup the map
        let map: CMap3<f64> = CMap3::new(5);
        map.force_link::<2>(1, 2).unwrap();
        map.force_link::<1>(4, 5).unwrap();
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

        let t1 = loom::thread::spawn(move || while let Err(_) = m1.force_sew::<1>(1, 3) {});

        let t2 = loom::thread::spawn(move || while let Err(_) = m2.force_sew::<2>(3, 4) {});

        t1.join().unwrap();
        t2.join().unwrap();

        // all paths should result in the same topological result here
        let v2 = arc.force_remove_vertex(2);
        let v3 = arc.force_remove_vertex(3);
        let v5 = arc.force_remove_vertex(5);
        assert!(v2.is_some());
        assert!(v3.is_none());
        assert!(v5.is_none());
        assert_eq!(Orbit3::new(arc.as_ref(), OrbitPolicy::Vertex, 2).count(), 3);
        assert!(arc.force_read_vertex(2).is_none());
        assert!(arc.force_read_vertex(3).is_none());
        assert!(arc.force_read_vertex(5).is_none());
    });
}

#[test]
fn sew_ordering_with_transactions() {
    loom::model(|| {
        // setup the map
        let map: CMap3<f64> = CMap3::new(5);
        map.force_link::<2>(1, 2).unwrap();
        map.force_link::<2>(3, 4).unwrap();
        // only one vertex is defined
        // the idea is to use CMapError, along with transaction control to ensure
        // we don't proceed with a sew on no value
        map.force_write_vertex(2, Vertex3(1.0, 1.0, 1.0));
        // map.force_write_vertex(3, Vertex3(1.0, 2.0, 1.0));
        // map.force_write_vertex(5, Vertex3(2.0, 2.0, 1.0));
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do two sew ops:
        // - 1-sew 1 to 3 (t1)
        // - 1-sew 4 to 5 (t2)
        // this will result in a single vertex being defined, of ID 2
        // to demonstrate order of execution, we're going to use a TVar
        let foo = TVar::new(0);
        let f = loom::sync::Arc::new(foo);
        let (f1, f2) = (f.clone(), f.clone());

        let t1 = loom::thread::spawn(move || {
            atomically(|trans| {
                f1.modify(trans, |v| v + 1)?;
                // this should be useless as the vertex is defined on this op
                // we still have to pattern match becaue CMapError cannot be automatically
                // coerced to StmError
                if let Err(e) = m1.sew::<1>(trans, 1, 3) {
                    match e {
                        TransactionError::Stm(e) => Err(e),
                        TransactionError::Abort(_) => Err(StmError::Retry),
                    }
                } else {
                    Ok(())
                }
            });
        });

        let t2 = loom::thread::spawn(move || {
            atomically(|trans| {
                f2.modify(trans, |v| if v != 0 { v + 4 } else { v })?;
                // if the first op landed, this won't create an error
                // otherwise, we'll either fail the transaction or fail the merge
                // in both (error) cases, we want to retry the transaction
                if let Err(e) = m2.sew::<1>(trans, 4, 5) {
                    match e {
                        TransactionError::Stm(e) => Err(e),
                        TransactionError::Abort(_) => Err(StmError::Retry),
                    }
                } else {
                    Ok(())
                }
            })
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // all paths should result in the same topological result here
        let (v2, v3, v5) = atomically(|trans| {
            Ok((
                arc.remove_vertex(trans, 2)?,
                arc.remove_vertex(trans, 3)?,
                arc.remove_vertex(trans, 5)?,
            ))
        });
        assert!(v2.is_some());
        assert!(v3.is_none());
        assert!(v5.is_none());
        assert_eq!(Orbit3::new(arc.as_ref(), OrbitPolicy::Vertex, 2).count(), 3);
        atomically(|trans| {
            assert!(arc.read_vertex(trans, 2)?.is_none());
            assert!(arc.read_vertex(trans, 3)?.is_none());
            assert!(arc.read_vertex(trans, 5)?.is_none());
            Ok(())
        });

        // if execution order was respected, foo should be at 5
        assert_eq!(f.read_atomic(), 5);
    });
}

#[derive(Debug, Clone, Copy, Default)]
struct Weight(pub u32);

impl AttributeUpdate for Weight {
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        Ok(Self(attr1.0 + attr2.0))
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        // adding the % to keep things conservative
        Ok((Weight(attr.0 / 2 + attr.0 % 2), Weight(attr.0 / 2)))
    }

    fn merge_incomplete(attr: Self) -> Result<Self, AttributeError> {
        Ok(attr)
    }
}

impl AttributeBind for Weight {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}

#[test]
fn unsew_ordering() {
    loom::model(|| {
        // setup the map FIXME: use the builder
        let mut map: CMap3<f64> = CMap3::new(5);
        map.attributes.add_storage::<Weight>(6);

        map.force_link::<2>(1, 2).unwrap();
        map.force_link::<2>(3, 4).unwrap();
        map.force_link::<1>(1, 3).unwrap();
        map.force_link::<1>(4, 5).unwrap();
        map.force_write_vertex(2, Vertex3(0.0, 0.0, 0.0));
        map.force_write_attribute(2, Weight(33));
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to unsew ops:
        // - 1-unsew 1 and 3 (t1)
        // - 2-unsew 3 and 4 (t2)
        // this will result in different weights, defined on IDs 2, 3, and 5

        let t1 = loom::thread::spawn(move || while let Err(_) = m1.force_unsew::<1>(1) {});

        let t2 = loom::thread::spawn(move || while let Err(_) = m2.force_unsew::<2>(3) {});

        t1.join().unwrap();
        t2.join().unwrap();

        // all paths should result in the same topological result here

        // We don't check for exact values here as they might differ based on execution order
        let w2 = arc.force_remove_attribute::<Weight>(2);
        let w3 = arc.force_remove_attribute::<Weight>(3);
        let w5 = arc.force_remove_attribute::<Weight>(5);
        assert!(w2.is_some());
        assert!(w3.is_some());
        assert!(w5.is_some());
        assert!(arc.force_read_attribute::<Weight>(2).is_none());
        assert!(arc.force_read_attribute::<Weight>(3).is_none());
        assert!(arc.force_read_attribute::<Weight>(5).is_none());
    });
}

#[test]
fn unsew_ordering_with_transactions() {
    loom::model(|| {
        // setup the map FIXME: use the builder
        let mut map: CMap3<f64> = CMap3::new(5);
        map.attributes.add_storage::<Weight>(6);

        let res = atomically_with_err(|trans| {
            map.link::<2>(trans, 1, 2)?;
            map.link::<2>(trans, 3, 4)?;
            map.link::<1>(trans, 1, 3)?;
            map.link::<1>(trans, 4, 5)?;
            map.write_vertex(trans, 2, (0.0, 0.0, 0.0))?;
            map.write_attribute(trans, 2, Weight(33))?;
            Ok(())
        });
        assert!(res.is_ok());
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to unsew ops:
        // - 1-unsew 1 and 3 (t1)
        // - 2-unsew 3 and 4 (t2)
        // this will result in different weights, defined on IDs 2, 3, and 5

        let t1 = loom::thread::spawn(move || {
            atomically(|trans| {
                if let Err(e) = m1.unsew::<1>(trans, 1) {
                    match e {
                        TransactionError::Stm(e) => Err(e),
                        TransactionError::Abort(_) => Err(StmError::Retry),
                    }
                } else {
                    Ok(())
                }
            });
        });

        let t2 = loom::thread::spawn(move || {
            atomically(|trans| {
                if let Err(e) = m2.unsew::<2>(trans, 3) {
                    match e {
                        TransactionError::Stm(e) => Err(e),
                        TransactionError::Abort(_) => Err(StmError::Retry),
                    }
                } else {
                    Ok(())
                }
            });
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // all paths should result in the same topological result here

        // We don't check for exact values here as they might differ based on execution order
        let (w2, w3, w5) = atomically(|trans| {
            Ok((
                arc.remove_attribute::<Weight>(trans, 2)?,
                arc.remove_attribute::<Weight>(trans, 3)?,
                arc.remove_attribute::<Weight>(trans, 5)?,
            ))
        });
        assert!(w2.is_some());
        assert!(w3.is_some());
        assert!(w5.is_some());
        atomically(|trans| {
            assert!(arc.read_attribute::<Weight>(trans, 2)?.is_none());
            assert!(arc.read_attribute::<Weight>(trans, 3)?.is_none());
            assert!(arc.read_attribute::<Weight>(trans, 5)?.is_none());
            Ok(())
        });
    });
}
