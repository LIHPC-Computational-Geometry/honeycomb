use anyhow::Context;

use crate::{
    attributes::{AttrSparseVec, AttributeBind, AttributeError, AttributeUpdate},
    cmap::{CMap3, CMapBuilder, DartIdType, OrbitPolicy, SewError, VertexIdType},
    geometry::Vertex3,
    stm::{StmError, TVar, TransactionError, atomically, atomically_with_err},
};

// allows returning a map in test functions
// not running checks on the state of the map since we may use minimal setup
// to cover some methods, which do not translate to correct mesh states
#[cfg(test)]
impl std::process::Termination for CMap3<f64> {
    fn report(self) -> std::process::ExitCode {
        std::process::ExitCode::SUCCESS
    }
}

// --- High-level tests

// force_* methods

#[test]
fn build_tet() -> anyhow::Result<CMap3<f64>> {
    let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(12).build()?; // 3*4 darts

    // face z- (base)
    map.force_link::<1>(1, 2)?;
    map.force_link::<1>(2, 3)?;
    map.force_link::<1>(3, 1)?;
    // face y-
    map.force_link::<1>(4, 5)?;
    map.force_link::<1>(5, 6)?;
    map.force_link::<1>(6, 4)?;
    // face x-
    map.force_link::<1>(7, 8)?;
    map.force_link::<1>(8, 9)?;
    map.force_link::<1>(9, 7)?;
    // face x+/y+
    map.force_link::<1>(10, 11)?;
    map.force_link::<1>(11, 12)?;
    map.force_link::<1>(12, 10)?;
    // link triangles to get the tet
    map.force_link::<2>(1, 4)?;
    map.force_link::<2>(2, 7)?;
    map.force_link::<2>(3, 10)?;
    map.force_link::<2>(5, 12)?;
    map.force_link::<2>(6, 8)?;
    map.force_link::<2>(9, 11)?;

    {
        let mut vertices = map.iter_vertices();
        assert_eq!(vertices.next(), Some(1));
        assert_eq!(vertices.next(), Some(2));
        assert_eq!(vertices.next(), Some(3));
        assert_eq!(vertices.next(), Some(6));
        assert_eq!(vertices.next(), None);

        let darts: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 2).collect();
        assert_eq!(&darts, &[2, 3, 1]);
    }

    map.force_write_vertex(1, (1.0, 0.0, 0.0));
    map.force_write_vertex(2, (0.0, 0.0, 0.0));
    map.force_write_vertex(3, (0.0, 0.5, 0.0));
    map.force_write_vertex(6, (0.5, 0.25, 1.0));

    Ok(map)
}

#[test]
fn sew_tets() -> anyhow::Result<CMap3<f64>> {
    // Build a tetrahedron (A)
    let mut map = build_tet().context("Failed to build first tetrahedron")?;

    // Build a second tetrahedron (B)

    let _ = map.allocate_used_darts(12);
    // face z- (base)
    map.force_link::<1>(13, 14)?;
    map.force_link::<1>(14, 15)?;
    map.force_link::<1>(15, 13)?;
    // face x-/y-
    map.force_link::<1>(16, 17)?;
    map.force_link::<1>(17, 18)?;
    map.force_link::<1>(18, 16)?;
    // face y+
    map.force_link::<1>(19, 20)?;
    map.force_link::<1>(20, 21)?;
    map.force_link::<1>(21, 19)?;
    // face x+
    map.force_link::<1>(22, 23)?;
    map.force_link::<1>(23, 24)?;
    map.force_link::<1>(24, 22)?;
    // link triangles to get the tet
    map.force_link::<2>(13, 16)?;
    map.force_link::<2>(14, 19)?;
    map.force_link::<2>(15, 22)?;
    map.force_link::<2>(17, 24)?;
    map.force_link::<2>(18, 20)?;
    map.force_link::<2>(21, 23)?;

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
        let mut edges = map.iter_edges();
        assert_eq!(edges.next(), Some(1));
        assert_eq!(edges.next(), Some(2));
        assert_eq!(edges.next(), Some(3));
        assert_eq!(edges.next(), Some(5));
        assert_eq!(edges.next(), Some(6));
        assert_eq!(edges.next(), Some(9));
        assert_eq!(edges.next(), Some(13));
        assert_eq!(edges.next(), Some(14));
        assert_eq!(edges.next(), Some(15));
        assert_eq!(edges.next(), Some(17));
        assert_eq!(edges.next(), Some(18));
        assert_eq!(edges.next(), Some(21));
        assert_eq!(edges.next(), None);
    }

    // Sew both tetrahedrons along a face (C)

    assert_eq!(map.n_vertices(), 8);
    map.force_sew::<3>(10, 16)?;
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

        let darts: Vec<_> = map.orbit(OrbitPolicy::Face, 10).collect();
        assert!(darts.contains(&10));
        assert!(darts.contains(&11));
        assert!(darts.contains(&12));
        assert!(darts.contains(&16));
        assert!(darts.contains(&17));
        assert!(darts.contains(&18));
    }

    Ok(map)
}

#[test]
fn unsew_tets() -> anyhow::Result<()> {
    let map = sew_tets().context("Failed to sew first tetrahedra")?;

    // this should get us back to the state before the first 3-sew
    map.force_unsew::<3>(10)?;
    assert_eq!(map.n_vertices(), 8);

    {
        let mut vertices = map.iter_vertices();
        assert_eq!(vertices.next(), Some(1));
        assert_eq!(vertices.next(), Some(2));
        assert_eq!(vertices.next(), Some(3));
        assert_eq!(vertices.next(), Some(6));
        assert_eq!(vertices.next(), Some(13));
        assert_eq!(vertices.next(), Some(14));
        assert_eq!(vertices.next(), Some(15));
        assert_eq!(vertices.next(), Some(18));
        assert_eq!(vertices.next(), None);

        let darts: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 2).collect();
        assert_eq!(&darts, &[2, 3, 1]);
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
        let mut edges = map.iter_edges();
        assert_eq!(edges.next(), Some(1));
        assert_eq!(edges.next(), Some(2));
        assert_eq!(edges.next(), Some(3));
        assert_eq!(edges.next(), Some(5));
        assert_eq!(edges.next(), Some(6));
        assert_eq!(edges.next(), Some(9));
        assert_eq!(edges.next(), Some(13));
        assert_eq!(edges.next(), Some(14));
        assert_eq!(edges.next(), Some(15));
        assert_eq!(edges.next(), Some(17));
        assert_eq!(edges.next(), Some(18));
        assert_eq!(edges.next(), Some(21));
        assert_eq!(edges.next(), None);

        let darts: Vec<_> = map.orbit(OrbitPolicy::Face, 10).collect();
        assert!(darts.len() == 3);
        assert!(darts.contains(&10));
        assert!(darts.contains(&11));
        assert!(darts.contains(&12));
        let darts: Vec<_> = map.orbit(OrbitPolicy::Face, 16).collect();
        assert!(darts.len() == 3);
        assert!(darts.contains(&16));
        assert!(darts.contains(&17));
        assert!(darts.contains(&18));
    }

    Ok(())
}

#[test]
fn merge_tets_into_pyramid() -> anyhow::Result<()> {
    fn rebuild_edge(map: &CMap3<f64>, dart: DartIdType) -> anyhow::Result<()> {
        let b3d = map.beta::<3>(dart);
        let ld = map.beta::<2>(dart);
        let rd = map.beta::<2>(b3d);

        map.force_unsew::<2>(dart)?;
        map.force_unsew::<2>(b3d)?;
        map.force_sew::<2>(ld, rd)?;
        Ok(())
    }

    let map = sew_tets().context("Failed to sew first tetrahedra")?;

    // Adjust shared vertices (D) to make a symmetrical square-base pyramid
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
    rebuild_edge(&map, 10)?;
    rebuild_edge(&map, 11)?;
    rebuild_edge(&map, 12)?;

    // delete old face components
    map.force_unlink::<1>(10)?;
    map.force_unlink::<1>(11)?;
    map.force_unlink::<1>(12)?;
    map.force_unlink::<3>(10)?;
    map.force_unlink::<3>(11)?;
    map.force_unlink::<3>(12)?;
    map.release_dart(10)?;
    map.release_dart(11)?;
    map.release_dart(12)?;
    map.release_dart(16)?;
    map.release_dart(17)?;
    map.release_dart(18)?;

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

    Ok(())
}

// transactional methods

#[test]
fn build_tet_tx() -> anyhow::Result<CMap3<f64>> {
    // Build a tetrahedron (A)
    let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(12).build().unwrap(); // 3*4 darts

    // face z- (base)
    let res = atomically_with_err(|t| {
        map.link::<1>(t, 1, 2)?;
        map.link::<1>(t, 2, 3)?;
        map.link::<1>(t, 3, 1)?;
        // face y-
        map.link::<1>(t, 4, 5)?;
        map.link::<1>(t, 5, 6)?;
        map.link::<1>(t, 6, 4)?;
        // face x-
        map.link::<1>(t, 7, 8)?;
        map.link::<1>(t, 8, 9)?;
        map.link::<1>(t, 9, 7)?;
        // face x+/y+
        map.link::<1>(t, 10, 11)?;
        map.link::<1>(t, 11, 12)?;
        map.link::<1>(t, 12, 10)?;
        // link triangles to get the tet
        map.link::<2>(t, 1, 4)?;
        map.link::<2>(t, 2, 7)?;
        map.link::<2>(t, 3, 10)?;
        map.link::<2>(t, 5, 12)?;
        map.link::<2>(t, 6, 8)?;
        map.link::<2>(t, 9, 11)?;
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

        let darts: Vec<_> = atomically(|t| {
            Ok(map
                .orbit_tx(t, OrbitPolicy::FaceLinear, 2)
                .map(Result::unwrap)
                .collect())
        });
        assert_eq!(&darts, &[2, 3, 1]);
    }

    atomically(|t| {
        map.write_vertex(t, 1, (1.0, 0.0, 0.0))?;
        map.write_vertex(t, 2, (0.0, 0.0, 0.0))?;
        map.write_vertex(t, 3, (0.0, 0.5, 0.0))?;
        map.write_vertex(t, 6, (0.5, 0.25, 1.0))?;
        Ok(())
    });

    Ok(map)
}

#[test]
fn sew_tets_tx() -> anyhow::Result<CMap3<f64>> {
    // Build a tetrahedron (A)
    let mut map = build_tet_tx().context("Failed to build first tetrahedron")?;

    // Build a second tetrahedron (B)
    let _ = map.allocate_used_darts(12);
    atomically_with_err(|t| {
        // face z- (base)
        map.link::<1>(t, 13, 14)?;
        map.link::<1>(t, 14, 15)?;
        map.link::<1>(t, 15, 13)?;
        // face x-/y-
        map.link::<1>(t, 16, 17)?;
        map.link::<1>(t, 17, 18)?;
        map.link::<1>(t, 18, 16)?;
        // face y+
        map.link::<1>(t, 19, 20)?;
        map.link::<1>(t, 20, 21)?;
        map.link::<1>(t, 21, 19)?;
        // face x+
        map.link::<1>(t, 22, 23)?;
        map.link::<1>(t, 23, 24)?;
        map.link::<1>(t, 24, 22)?;
        // link triangles to get the tet
        map.link::<2>(t, 13, 16)?;
        map.link::<2>(t, 14, 19)?;
        map.link::<2>(t, 15, 22)?;
        map.link::<2>(t, 17, 24)?;
        map.link::<2>(t, 18, 20)?;
        map.link::<2>(t, 21, 23)?;

        map.write_vertex(t, 13, (2.5, 1.5, 0.0))?;
        map.write_vertex(t, 14, (1.5, 2.0, 0.0))?;
        map.write_vertex(t, 15, (2.5, 2.0, 0.0))?;
        map.write_vertex(t, 18, (1.5, 1.75, 1.0))?;
        Ok(())
    })?;

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
        let mut edges = map.iter_edges();
        assert_eq!(edges.next(), Some(1));
        assert_eq!(edges.next(), Some(2));
        assert_eq!(edges.next(), Some(3));
        assert_eq!(edges.next(), Some(5));
        assert_eq!(edges.next(), Some(6));
        assert_eq!(edges.next(), Some(9));
        assert_eq!(edges.next(), Some(13));
        assert_eq!(edges.next(), Some(14));
        assert_eq!(edges.next(), Some(15));
        assert_eq!(edges.next(), Some(17));
        assert_eq!(edges.next(), Some(18));
        assert_eq!(edges.next(), Some(21));
        assert_eq!(edges.next(), None);
    }

    // Sew both tetrahedrons along a face (C)
    assert_eq!(map.n_vertices(), 8);
    atomically_with_err(|t| map.sew::<3>(t, 10, 16))?;
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

        let darts: Vec<_> = atomically(|t| {
            Ok(map
                .orbit_tx(t, OrbitPolicy::Face, 10)
                .map(Result::unwrap)
                .collect())
        });
        assert!(darts.contains(&10));
        assert!(darts.contains(&11));
        assert!(darts.contains(&12));
        assert!(darts.contains(&16));
        assert!(darts.contains(&17));
        assert!(darts.contains(&18));
    }

    Ok(map)
}

#[test]
fn unsew_tets_tx() -> anyhow::Result<()> {
    let map = sew_tets_tx().context("Failed to sew first tetrahedra")?;

    // this should get us back to the state before the first 3-sew
    atomically_with_err(|t| map.unsew::<3>(t, 10))?;
    assert_eq!(map.n_vertices(), 8);

    {
        let mut vertices = map.iter_vertices();
        assert_eq!(vertices.next(), Some(1));
        assert_eq!(vertices.next(), Some(2));
        assert_eq!(vertices.next(), Some(3));
        assert_eq!(vertices.next(), Some(6));
        assert_eq!(vertices.next(), Some(13));
        assert_eq!(vertices.next(), Some(14));
        assert_eq!(vertices.next(), Some(15));
        assert_eq!(vertices.next(), Some(18));
        assert_eq!(vertices.next(), None);

        let darts: Vec<_> = atomically(|t| {
            let mut tmp = Vec::with_capacity(3);
            for d in map.orbit_tx(t, OrbitPolicy::FaceLinear, 2) {
                tmp.push(d?);
            }
            Ok(tmp)
        });
        assert_eq!(&darts, &[2, 3, 1]);
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
        let mut edges = map.iter_edges();
        assert_eq!(edges.next(), Some(1));
        assert_eq!(edges.next(), Some(2));
        assert_eq!(edges.next(), Some(3));
        assert_eq!(edges.next(), Some(5));
        assert_eq!(edges.next(), Some(6));
        assert_eq!(edges.next(), Some(9));
        assert_eq!(edges.next(), Some(13));
        assert_eq!(edges.next(), Some(14));
        assert_eq!(edges.next(), Some(15));
        assert_eq!(edges.next(), Some(17));
        assert_eq!(edges.next(), Some(18));
        assert_eq!(edges.next(), Some(21));
        assert_eq!(edges.next(), None);

        let darts: Vec<_> = atomically(|t| {
            let mut tmp = Vec::with_capacity(3);
            for d in map.orbit_tx(t, OrbitPolicy::Face, 10) {
                tmp.push(d?);
            }
            Ok(tmp)
        });
        assert!(darts.len() == 3);
        assert!(darts.contains(&10));
        assert!(darts.contains(&11));
        assert!(darts.contains(&12));
        let darts: Vec<_> = atomically(|t| {
            let mut tmp = Vec::with_capacity(3);
            for d in map.orbit_tx(t, OrbitPolicy::Face, 16) {
                tmp.push(d?);
            }
            Ok(tmp)
        });
        assert!(darts.len() == 3);
        assert!(darts.contains(&16));
        assert!(darts.contains(&17));
        assert!(darts.contains(&18));
    }

    Ok(())
}

#[test]
fn merge_tets_into_pyramid_tx() -> anyhow::Result<()> {
    fn rebuild_edge(map: &CMap3<f64>, dart: DartIdType) -> anyhow::Result<()> {
        atomically_with_err(|t| {
            let b3d = map.beta_tx::<3>(t, dart)?;
            let ld = map.beta_tx::<2>(t, dart)?;
            let rd = map.beta_tx::<2>(t, b3d)?;

            map.unsew::<2>(t, dart)?;
            map.unsew::<2>(t, b3d)?;
            map.sew::<2>(t, ld, rd)?;
            Ok(())
        })?;
        Ok(())
    }

    let map = sew_tets_tx().context("Failed to sew first tetrahedra")?;

    // Adjust shared vertices (D)
    atomically(|t| {
        // this makes it a symmetrical square-base pyramid
        assert_eq!(
            map.write_vertex(t, 3, (0.0, 1.0, 0.0))?,
            Some(Vertex3(0.75, 1.25, 0.0))
        );
        assert_eq!(
            map.write_vertex(t, 1, (1.0, 0.0, 0.0))?,
            Some(Vertex3(1.75, 0.75, 0.0))
        );
        assert_eq!(
            map.write_vertex(t, 6, (0.5, 0.5, 1.0))?,
            Some(Vertex3(1.0, 1.0, 1.0))
        );
        assert_eq!(
            map.write_vertex(t, 15, (1.0, 1.0, 0.0))?,
            Some(Vertex3(2.5, 2.0, 0.0))
        );
        Ok(())
    });

    // Remove the split to have a single volume pyramid (E)

    rebuild_edge(&map, 10)?;
    rebuild_edge(&map, 11)?;
    rebuild_edge(&map, 12)?;

    // delete old face components
    atomically_with_err(|t| {
        map.unlink::<1>(t, 10)?;
        map.unlink::<1>(t, 11)?;
        map.unlink::<1>(t, 12)?;
        map.unlink::<3>(t, 10)?;
        map.unlink::<3>(t, 11)?;
        map.unlink::<3>(t, 12)?;
        Ok(())
    })?;

    atomically_with_err(|t| {
        map.release_dart_tx(t, 10)?;
        map.release_dart_tx(t, 11)?;
        map.release_dart_tx(t, 12)?;
        map.release_dart_tx(t, 16)?;
        map.release_dart_tx(t, 17)?;
        map.release_dart_tx(t, 18)?;
        Ok(())
    })?;

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

    Ok(())
}

// --- basic ops

#[test]
fn reserve_darts() {
    let mut map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(1).build().unwrap();
    map.allocate_unused_darts(100);
    assert!(
        map.reserve_darts(10)
            .is_ok_and(|s| s.len() == 10 && *s.first().unwrap() == 2 && *s.last().unwrap() == 11)
    );
    assert!(map.reserve_darts(100).is_err());
    assert!(
        atomically_with_err(|t| map.reserve_darts_from_tx(t, 10, 40))
            .is_ok_and(|s| s.len() == 10 && *s.first().unwrap() == 40 && *s.last().unwrap() == 49)
    );
}

#[test]
fn remove_vertex_twice() {
    let map: CMap3<f64> = CMap3::new(4);
    assert!(map.force_write_vertex(1, (1.0, 1.0, 1.0)).is_none());
    assert_eq!(map.force_remove_vertex(1), Some(Vertex3(1.0, 1.0, 1.0)));
    assert!(map.force_remove_vertex(1).is_none());
}

#[test]
fn remove_dart_twice() {
    // in its default state, all darts are:
    // - used
    // - free
    let map: CMap3<f64> = CMap3::new(4);
    // set dart 1 as unused
    assert!(!map.release_dart(1).unwrap());
    // set dart 1 as unused, again
    assert!(map.release_dart(1).unwrap());
}

// --- (un)sew

mod one_sew {
    use crate::cmap::LinkError;

    use super::*;

    // topology

    #[test]
    fn topo_no_b3_image() -> anyhow::Result<()> {
        let map: CMap3<f64> = CMap3::new(8);
        // map.force_link::<1>(1, 2);
        map.force_link::<1>(2, 3)?;
        map.force_link::<1>(3, 4)?;
        map.force_link::<1>(4, 1)?;
        map.force_write_vertex(1, Vertex3(0.0, 0.0, 0.0));
        map.force_write_vertex(2, Vertex3(1.0, 0.0, 0.0));
        map.force_write_vertex(3, Vertex3(1.0, 1.0, 0.0));
        map.force_write_vertex(4, Vertex3(0.0, 1.0, 0.0));

        map.force_sew::<1>(1, 2)?;

        assert_eq!(
            &[3, 4, 1, 2],
            map.orbit(OrbitPolicy::FaceLinear, 3)
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(map.beta::<1>(1), 2);
        assert_eq!(map.beta::<0>(2), 1);

        Ok(())
    }

    #[test]
    fn topo_b3_image() -> anyhow::Result<()> {
        let map: CMap3<f64> = CMap3::new(8);
        // map.force_link::<1>(1, 2);
        map.force_link::<1>(2, 3)?;
        map.force_link::<1>(3, 4)?;
        map.force_link::<1>(4, 1)?;
        map.force_write_vertex(1, Vertex3(0.0, 0.0, 0.0));
        map.force_write_vertex(2, Vertex3(1.0, 0.0, 0.0));
        map.force_write_vertex(3, Vertex3(1.0, 1.0, 0.0));
        map.force_write_vertex(4, Vertex3(0.0, 1.0, 0.0));

        map.force_link::<1>(5, 6)?;
        map.force_link::<1>(6, 7)?;
        map.force_link::<1>(7, 8)?;
        // map.force_link::<1>(8, 5);
        map.force_write_vertex(5, Vertex3(0.5, 0.0, 1.0));
        map.force_write_vertex(6, Vertex3(0.0, 0.0, 1.0));
        map.force_write_vertex(7, Vertex3(0.0, 1.0, 1.0));
        map.force_write_vertex(8, Vertex3(1.0, 1.0, 1.0));

        map.force_sew::<3>(1, 5)?;
        assert_eq!(map.beta::<3>(1), 5);
        assert_eq!(map.beta::<3>(2), 8);
        assert_eq!(map.beta::<3>(3), 7);
        assert_eq!(map.beta::<3>(4), 6);

        map.force_sew::<1>(1, 2)?;

        assert_eq!(map.beta::<1>(1), 2);
        assert_eq!(map.beta::<1>(8), 5);
        assert_eq!(map.vertex_id(5), 2);
        assert_eq!(map.force_read_vertex(2), Some(Vertex3(0.75, 0.0, 0.5)));

        Ok(())
    }

    #[test]
    fn topo_errs() -> anyhow::Result<()> {
        // 1-sew unfree dart
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(3).build()?;
            map.force_link::<1>(1, 2)?;

            assert!(
                map.force_sew::<1>(1, 3).is_err_and(|e| matches!(
                    e,
                    SewError::FailedLink(LinkError::NonFreeBase(1, 1, 3))
                ))
            )
        }
        // 1-sew to unfree dart
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(3).build()?;
            map.force_link::<1>(2, 3)?;

            assert!(map.force_sew::<1>(1, 3).is_err_and(|e| matches!(
                e,
                SewError::FailedLink(LinkError::NonFreeImage(0, 1, 3))
            )))
        }
        // 1-sew to null dart
        {
            // FIXME: implement a check?
        }
        // 1-sew to dart with incompatible b3 topology
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(5).build()?;
            map.force_link::<3>(1, 2)?;
            map.force_link::<3>(3, 4)?;
            map.force_link::<1>(4, 5)?;
            map.force_write_vertex(2, (0.0, 0.0, 0.0));
            map.force_write_vertex(3, (0.0, 1.0, 0.0));

            assert!(map.force_sew::<1>(1, 3).is_err_and(|e| matches!(
                e,
                SewError::FailedLink(LinkError::NonFreeBase(1, 4, 2))
            )));
        }
        Ok(())
    }

    // geometry
    #[test]
    fn geom_no_orbit() -> anyhow::Result<()> {
        let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(2).build()?;
        map.force_write_vertex(2, (0.0, 1.0, 0.0));

        atomically_with_err(|t| map.sew::<1>(t, 1, 2))?;

        atomically(|t| {
            assert_eq!(map.vertex_id_tx(t, 1)?, 1);
            assert_eq!(map.read_vertex(t, 1)?, None);
            assert_eq!(map.vertex_id_tx(t, 2)?, 2);
            assert_eq!(map.read_vertex(t, 2)?, Some(Vertex3(0.0, 1.0, 0.0)));
            Ok(())
        });

        Ok(())
    }

    #[test]
    fn geom_no_b2_image() -> anyhow::Result<()> {
        // new vid == rd == 2
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(3).build()?;
            map.force_link::<3>(1, 3)?;
            map.force_write_vertex(2, (0.0, 1.0, 0.0));
            map.force_write_vertex(3, (0.0, 0.0, 0.0));

            atomically_with_err(|t| map.sew::<1>(t, 1, 2))?;

            atomically(|t| {
                assert_eq!(map.vertex_id_tx(t, 1)?, 1);
                assert_eq!(map.read_vertex(t, 1)?, None);
                assert_eq!(map.vertex_id_tx(t, 2)?, 2);
                assert_eq!(map.vertex_id_tx(t, 3)?, 2);
                assert_eq!(map.read_vertex(t, 2)?, Some(Vertex3(0.0, 0.5, 0.0)));
                Ok(())
            });
        }
        // new vid == b3ld == 2
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(3).build()?;
            map.force_link::<3>(1, 2)?;
            map.force_write_vertex(2, (0.0, 0.0, 0.0));
            map.force_write_vertex(3, (0.0, 1.0, 0.0));

            atomically_with_err(|t| map.sew::<1>(t, 1, 3))?;

            atomically(|t| {
                assert_eq!(map.vertex_id_tx(t, 1)?, 1);
                assert_eq!(map.read_vertex(t, 1)?, None);
                assert_eq!(map.vertex_id_tx(t, 2)?, 2);
                assert_eq!(map.vertex_id_tx(t, 3)?, 2);
                assert_eq!(map.read_vertex(t, 2)?, Some(Vertex3(0.0, 0.5, 0.0)));
                Ok(())
            });
        }

        Ok(())
    }

    #[test]
    fn geom_no_b3_image() -> anyhow::Result<()> {
        // new vid == rd == 2
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(3).build()?;
            map.force_link::<2>(1, 3)?;
            map.force_write_vertex(2, (0.0, 1.0, 0.0));
            map.force_write_vertex(3, (0.0, 0.0, 0.0));

            atomically_with_err(|t| map.sew::<1>(t, 1, 2))?;

            atomically(|t| {
                assert_eq!(map.vertex_id_tx(t, 1)?, 1);
                assert_eq!(map.read_vertex(t, 1)?, None);
                assert_eq!(map.vertex_id_tx(t, 2)?, 2);
                assert_eq!(map.vertex_id_tx(t, 3)?, 2);
                assert_eq!(map.read_vertex(t, 2)?, Some(Vertex3(0.0, 0.5, 0.0)));
                Ok(())
            });
        }
        // new vid == b2ld == 2
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(3).build()?;
            map.force_link::<2>(1, 2)?;
            map.force_write_vertex(2, (0.0, 0.0, 0.0));
            map.force_write_vertex(3, (0.0, 1.0, 0.0));

            atomically_with_err(|t| map.sew::<1>(t, 1, 3))?;

            atomically(|t| {
                assert_eq!(map.vertex_id_tx(t, 1)?, 1);
                assert_eq!(map.read_vertex(t, 1)?, None);
                assert_eq!(map.vertex_id_tx(t, 2)?, 2);
                assert_eq!(map.vertex_id_tx(t, 3)?, 2);
                assert_eq!(map.read_vertex(t, 2)?, Some(Vertex3(0.0, 0.5, 0.0)));
                Ok(())
            });
        }

        Ok(())
    }

    #[test]
    fn geom_full_orbit() -> anyhow::Result<()> {
        // new vid == rd == 2
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(4).build()?;
            map.force_link::<2>(1, 3)?;
            map.force_link::<3>(1, 4)?;
            assert_eq!(map.vertex_id(3), 3);
            assert_eq!(map.vertex_id(4), 3);
            map.force_write_vertex(2, (0.0, 1.0, 0.0));
            map.force_write_vertex(3, (0.0, 0.0, 0.0));

            atomically_with_err(|t| map.sew::<1>(t, 1, 2))?;

            atomically(|t| {
                assert_eq!(map.vertex_id_tx(t, 1)?, 1);
                assert_eq!(map.read_vertex(t, 1)?, None);
                assert_eq!(map.vertex_id_tx(t, 2)?, 2);
                assert_eq!(map.vertex_id_tx(t, 3)?, 2);
                assert_eq!(map.vertex_id_tx(t, 4)?, 2);
                assert_eq!(map.read_vertex(t, 2)?, Some(Vertex3(0.0, 0.5, 0.0)));
                Ok(())
            });
        }
        // new vid == b2ld == 2
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(4).build()?;
            map.force_link::<2>(1, 2)?;
            map.force_link::<3>(1, 3)?;
            assert_eq!(map.vertex_id(2), 2);
            assert_eq!(map.vertex_id(3), 2);
            map.force_write_vertex(2, (0.0, 0.0, 0.0));
            map.force_write_vertex(4, (0.0, 1.0, 0.0));

            atomically_with_err(|t| map.sew::<1>(t, 1, 4))?;

            atomically(|t| {
                assert_eq!(map.vertex_id_tx(t, 1)?, 1);
                assert_eq!(map.read_vertex(t, 1)?, None);
                assert_eq!(map.vertex_id_tx(t, 2)?, 2);
                assert_eq!(map.vertex_id_tx(t, 3)?, 2);
                assert_eq!(map.vertex_id_tx(t, 4)?, 2);
                assert_eq!(map.read_vertex(t, 2)?, Some(Vertex3(0.0, 0.5, 0.0)));
                Ok(())
            });
        }
        // new vid == b3ld == 2
        {
            let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(4).build()?;
            map.force_link::<2>(1, 4)?;
            map.force_link::<3>(1, 2)?;
            assert_eq!(map.vertex_id(2), 2);
            assert_eq!(map.vertex_id(4), 2);
            map.force_write_vertex(2, (0.0, 0.0, 0.0));
            map.force_write_vertex(3, (0.0, 1.0, 0.0));

            atomically_with_err(|t| map.sew::<1>(t, 1, 3))?;

            atomically(|t| {
                assert_eq!(map.vertex_id_tx(t, 1)?, 1);
                assert_eq!(map.read_vertex(t, 1)?, None);
                assert_eq!(map.vertex_id_tx(t, 2)?, 2);
                assert_eq!(map.vertex_id_tx(t, 3)?, 2);
                assert_eq!(map.vertex_id_tx(t, 4)?, 2);
                assert_eq!(map.read_vertex(t, 2)?, Some(Vertex3(0.0, 0.5, 0.0)));
                Ok(())
            });
        }

        Ok(())
    }

    // NOTE: in the case where ld has no vertex orbit at its end, a sew with no attribute data
    // may be tolerated, even if rd has an orbit with missing attribute
    #[test]
    fn geom_missing_data() -> anyhow::Result<()> {
        let map: CMap3<f64> = CMapBuilder::<3>::from_n_darts(4).build()?;
        map.force_link::<2>(1, 3)?;

        assert!(
            atomically_with_err(|t| map.sew::<1>(t, 1, 2)).is_err_and(|e| matches!(
                e,
                SewError::FailedAttributeOp(AttributeError::InsufficientData(_, _))
            ))
        );

        map.force_unlink::<2>(1)?;
        map.force_link::<3>(1, 4)?;

        assert!(
            atomically_with_err(|t| map.sew::<1>(t, 1, 2)).is_err_and(|e| matches!(
                e,
                SewError::FailedAttributeOp(AttributeError::InsufficientData(_, _))
            ))
        );

        map.force_link::<2>(1, 3)?;

        assert!(
            atomically_with_err(|t| map.sew::<1>(t, 1, 2)).is_err_and(|e| matches!(
                e,
                SewError::FailedAttributeOp(AttributeError::InsufficientData(_, _))
            ))
        );

        Ok(())
    }
}

mod two_sew {
    use super::*;

    // topology

    #[test]
    fn topo_no_b3_image() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn topo_b3_image() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn topo_errs() -> anyhow::Result<()> {
        // todo!()
        Ok(())
    }

    // geometry
    #[test]
    fn geom_no_orbit() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_no_b3_image() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_no_b1_image() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_full_orbit() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_missing_data() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_bad_orientation() -> anyhow::Result<()> {
        let map: CMap3<f64> = CMap3::new(8);
        atomically_with_err(|t| {
            map.link::<1>(t, 1, 2)?;
            map.link::<1>(t, 2, 3)?;
            map.link::<1>(t, 3, 4)?;
            map.link::<1>(t, 4, 1)?;
            map.link::<1>(t, 5, 6)?;
            map.link::<1>(t, 6, 7)?;
            map.link::<1>(t, 7, 8)?;
            map.link::<1>(t, 8, 5)?;
            map.write_vertex(t, 1, Vertex3(0.0, 0.0, 0.0))?;
            map.write_vertex(t, 2, Vertex3(1.0, 0.0, 0.0))?;
            map.write_vertex(t, 3, Vertex3(1.0, 1.0, 0.0))?;
            map.write_vertex(t, 4, Vertex3(0.0, 1.0, 0.0))?;
            map.write_vertex(t, 5, Vertex3(0.0, 0.0, 1.0))?;
            map.write_vertex(t, 6, Vertex3(1.0, 0.0, 1.0))?;
            map.write_vertex(t, 7, Vertex3(1.0, 1.0, 1.0))?;
            map.write_vertex(t, 8, Vertex3(0.0, 1.0, 1.0))?;
            Ok(())
        })?;

        assert!(
            map.force_sew::<2>(1, 5)
                .is_err_and(|e| e == SewError::BadGeometry(2, 1, 5))
        );

        Ok(())
    }
}

mod three_sew {
    use super::*;

    // topology

    #[test]
    fn topo_open_face_no_b2_image() -> anyhow::Result<()> {
        let map: CMap3<f64> = CMap3::new(8);
        atomically_with_err(|t| {
            // map.link::<1>(t, 1, 2)?;
            map.link::<1>(t, 2, 3)?;
            map.link::<1>(t, 3, 4)?;
            map.link::<1>(t, 4, 1)?;
            map.write_vertex(t, 1, Vertex3(0.0, 0.0, 0.0))?;
            map.write_vertex(t, 2, Vertex3(1.0, 0.0, 0.0))?;
            map.write_vertex(t, 3, Vertex3(1.0, 1.0, 0.0))?;
            map.write_vertex(t, 4, Vertex3(0.0, 1.0, 0.0))?;

            map.link::<1>(t, 5, 6)?;
            map.link::<1>(t, 6, 7)?;
            // map.link::<1>(t, 7, 8)?;
            map.link::<1>(t, 8, 5)?;
            map.write_vertex(t, 5, Vertex3(0.0, 0.0, 1.0))?;
            map.write_vertex(t, 6, Vertex3(0.0, 1.0, 1.0))?;
            map.write_vertex(t, 7, Vertex3(1.0, 1.0, 1.0))?;
            map.write_vertex(t, 8, Vertex3(1.0, 0.0, 1.0))?;

            Ok(())
        })?;

        atomically_with_err(|t| map.sew::<3>(t, 1, 8))?;

        assert_eq!(map.force_read_vertex(1).unwrap(), Vertex3(0.0, 0.0, 0.5));
        assert_eq!(map.force_read_vertex(2).unwrap(), Vertex3(1.0, 0.0, 0.0));
        assert_eq!(map.force_read_vertex(3).unwrap(), Vertex3(1.0, 1.0, 0.5));
        assert_eq!(map.force_read_vertex(4).unwrap(), Vertex3(0.0, 1.0, 0.5));
        assert_eq!(map.force_read_vertex(8).unwrap(), Vertex3(1.0, 0.0, 1.0));

        Ok(())
    }

    #[test]
    fn topo_closed_face_no_b2_image() -> anyhow::Result<()> {
        let map: CMap3<f64> = CMap3::new(8);
        atomically_with_err(|t| {
            map.link::<1>(t, 1, 2)?;
            map.link::<1>(t, 2, 3)?;
            map.link::<1>(t, 3, 4)?;
            map.link::<1>(t, 4, 1)?;
            map.write_vertex(t, 1, Vertex3(0.0, 0.0, 0.0))?;
            map.write_vertex(t, 2, Vertex3(1.0, 0.0, 0.0))?;
            map.write_vertex(t, 3, Vertex3(1.0, 1.0, 0.0))?;
            map.write_vertex(t, 4, Vertex3(0.0, 1.0, 0.0))?;

            map.link::<1>(t, 5, 6)?;
            map.link::<1>(t, 6, 7)?;
            map.link::<1>(t, 7, 8)?;
            map.link::<1>(t, 8, 5)?;
            map.write_vertex(t, 5, Vertex3(0.0, 0.0, 1.0))?;
            map.write_vertex(t, 6, Vertex3(0.0, 1.0, 1.0))?;
            map.write_vertex(t, 7, Vertex3(1.0, 1.0, 1.0))?;
            map.write_vertex(t, 8, Vertex3(1.0, 0.0, 1.0))?;

            Ok(())
        })?;

        atomically_with_err(|t| map.sew::<3>(t, 1, 8))?;

        assert_eq!(map.force_read_vertex(1).unwrap(), Vertex3(0.0, 0.0, 0.5));
        assert_eq!(map.force_read_vertex(2).unwrap(), Vertex3(1.0, 0.0, 0.5));
        assert_eq!(map.force_read_vertex(3).unwrap(), Vertex3(1.0, 1.0, 0.5));
        assert_eq!(map.force_read_vertex(4).unwrap(), Vertex3(0.0, 1.0, 0.5));

        Ok(())
    }

    #[test]
    fn topo_b2_image() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn topo_errs() -> anyhow::Result<()> {
        // todo!()
        Ok(())
    }

    // geometry
    #[test]
    fn geom_no_orbit() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_no_b2_image() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_no_b1_image() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_full_orbit() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_missing_data() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn geom_bad_orientation() -> anyhow::Result<()> {
        let map: CMap3<f64> = CMap3::new(8);
        atomically_with_err(|t| {
            map.link::<1>(t, 1, 2)?;
            map.link::<1>(t, 2, 3)?;
            map.link::<1>(t, 3, 4)?;
            map.link::<1>(t, 4, 1)?;
            map.link::<1>(t, 5, 6)?;
            map.link::<1>(t, 6, 7)?;
            map.link::<1>(t, 7, 8)?;
            map.link::<1>(t, 8, 5)?;
            map.write_vertex(t, 1, Vertex3(0.0, 0.0, 0.0))?;
            map.write_vertex(t, 2, Vertex3(1.0, 0.0, 0.0))?;
            map.write_vertex(t, 3, Vertex3(1.0, 1.0, 0.0))?;
            map.write_vertex(t, 4, Vertex3(0.0, 1.0, 0.0))?;
            map.write_vertex(t, 5, Vertex3(0.0, 0.0, 1.0))?;
            map.write_vertex(t, 6, Vertex3(1.0, 0.0, 1.0))?;
            map.write_vertex(t, 7, Vertex3(1.0, 1.0, 1.0))?;
            map.write_vertex(t, 8, Vertex3(0.0, 1.0, 1.0))?;
            Ok(())
        })?;

        assert!(
            map.force_sew::<3>(1, 5)
                .is_err_and(|e| e == SewError::BadGeometry(3, 1, 5))
        );

        Ok(())
    }
}

#[test]
fn three_sew() {}

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

        let t1 = loom::thread::spawn(move || while m1.force_sew::<1>(1, 3).is_err() {});

        let t2 = loom::thread::spawn(move || while m2.force_sew::<2>(3, 4).is_err() {});

        t1.join().unwrap();
        t2.join().unwrap();

        // all paths should result in the same topological result here
        let v2 = arc.force_remove_vertex(2);
        let v3 = arc.force_remove_vertex(3);
        let v5 = arc.force_remove_vertex(5);
        assert!(v2.is_some());
        assert!(v3.is_none());
        assert!(v5.is_none());
        assert_eq!(arc.orbit(OrbitPolicy::Vertex, 2).count(), 3);
        assert!(arc.force_read_vertex(2).is_none());
        assert!(arc.force_read_vertex(3).is_none());
        assert!(arc.force_read_vertex(5).is_none());
    });
}

#[test]
fn sew_ordering_with_txtions() {
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
            atomically(|t| {
                f1.modify(t, |v| v + 1)?;
                // this should be useless as the vertex is defined on this op
                // we still have to pattern match because CMapError cannot be automatically
                // coerced to StmError
                if let Err(e) = m1.sew::<1>(t, 1, 3) {
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
            atomically(|t| {
                f2.modify(t, |v| if v != 0 { v + 4 } else { v })?;
                // if the first op landed, this won't create an error
                // otherwise, we'll either fail the transaction or fail the merge
                // in both (error) cases, we want to retry the transaction
                if let Err(e) = m2.sew::<1>(t, 4, 5) {
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
        let (v2, v3, v5) = atomically(|t| {
            Ok((
                arc.remove_vertex(t, 2)?,
                arc.remove_vertex(t, 3)?,
                arc.remove_vertex(t, 5)?,
            ))
        });
        assert!(v2.is_some());
        assert!(v3.is_none());
        assert!(v5.is_none());
        assert_eq!(arc.orbit(OrbitPolicy::Vertex, 2).count(), 3);
        atomically(|t| {
            assert!(arc.read_vertex(t, 2)?.is_none());
            assert!(arc.read_vertex(t, 3)?.is_none());
            assert!(arc.read_vertex(t, 5)?.is_none());
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

        let t1 = loom::thread::spawn(move || while m1.force_unsew::<1>(1).is_err() {});

        let t2 = loom::thread::spawn(move || while m2.force_unsew::<2>(3).is_err() {});

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
fn unsew_ordering_with_txtions() {
    loom::model(|| {
        // setup the map FIXME: use the builder
        let mut map: CMap3<f64> = CMap3::new(5);
        map.attributes.add_storage::<Weight>(6);

        let res = atomically_with_err(|t| {
            map.link::<2>(t, 1, 2)?;
            map.link::<2>(t, 3, 4)?;
            map.link::<1>(t, 1, 3)?;
            map.link::<1>(t, 4, 5)?;
            map.write_vertex(t, 2, (0.0, 0.0, 0.0))?;
            map.write_attribute(t, 2, Weight(33))?;
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
            atomically(|t| {
                if let Err(e) = m1.unsew::<1>(t, 1) {
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
            atomically(|t| {
                if let Err(e) = m2.unsew::<2>(t, 3) {
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
        let (w2, w3, w5) = atomically(|t| {
            Ok((
                arc.remove_attribute::<Weight>(t, 2)?,
                arc.remove_attribute::<Weight>(t, 3)?,
                arc.remove_attribute::<Weight>(t, 5)?,
            ))
        });
        assert!(w2.is_some());
        assert!(w3.is_some());
        assert!(w5.is_some());
        atomically(|t| {
            assert!(arc.read_attribute::<Weight>(t, 2)?.is_none());
            assert!(arc.read_attribute::<Weight>(t, 3)?.is_none());
            assert!(arc.read_attribute::<Weight>(t, 5)?.is_none());
            Ok(())
        });
    });
}
