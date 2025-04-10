use std::collections::HashSet;

use honeycomb_core::{
    attributes::{AttrSparseVec, AttributeStorage, UnknownAttributeStorage},
    cmap::{CMap2, CMapBuilder, NULL_DART_ID, OrbitPolicy},
    stm::{atomically, atomically_with_err},
};

use crate::{
    grisubal::Clip,
    remeshing::{ClassificationError, EdgeSwapError, VertexAnchor, swap_edge},
};

use super::{EdgeAnchor, FaceAnchor, capture_geometry, classify_capture};

// --- anchors

#[test]
fn merge_vertex_eq_dim() {
    let storage: AttrSparseVec<VertexAnchor> = AttrSparseVec::new(13);
    atomically(|t| {
        storage.write(t, 1, VertexAnchor::Node(1))?;
        storage.write(t, 2, VertexAnchor::Node(1))?;
        storage.write(t, 3, VertexAnchor::Node(2))?;
        storage.write(t, 4, VertexAnchor::Curve(1))?;
        storage.write(t, 5, VertexAnchor::Curve(1))?;
        storage.write(t, 6, VertexAnchor::Curve(2))?;
        storage.write(t, 7, VertexAnchor::Surface(3))?;
        storage.write(t, 8, VertexAnchor::Surface(3))?;
        storage.write(t, 9, VertexAnchor::Surface(4))?;
        storage.write(t, 10, VertexAnchor::Body(5))?;
        storage.write(t, 11, VertexAnchor::Body(6))?;
        storage.write(t, 12, VertexAnchor::Body(6))?;
        Ok(())
    });

    atomically(|t| {
        // Node merges
        assert!(storage.merge(t, 1, 1, 2).is_ok());
        assert!(storage.read(t, 1)?.is_some());
        assert!(storage.read(t, 2)?.is_none());
        assert!(storage.merge(t, 1, 1, 3).is_err());
        assert!(storage.read(t, 3)?.is_some());
        // Curve merges
        assert!(storage.merge(t, 4, 4, 5).is_ok());
        assert!(storage.read(t, 4)?.is_some());
        assert!(storage.read(t, 5)?.is_none());
        assert!(storage.merge(t, 6, 4, 6).is_err());
        assert!(storage.read(t, 6)?.is_some());
        // Surface merges
        assert!(storage.merge(t, 7, 7, 8).is_ok());
        assert!(storage.read(t, 7)?.is_some());
        assert!(storage.read(t, 8)?.is_none());
        assert!(storage.merge(t, 7, 7, 9).is_err());
        assert!(storage.read(t, 9)?.is_some());
        // Body merges
        assert!(storage.merge(t, 11, 12, 11).is_ok());
        assert!(storage.read(t, 11)?.is_some());
        assert!(storage.read(t, 12)?.is_none());
        assert!(storage.merge(t, 10, 10, 11).is_err());
        assert!(storage.read(t, 11)?.is_some());
        Ok(())
    });
}

#[test]
fn merge_vertex_diff_dim() {
    let storage: AttrSparseVec<VertexAnchor> = AttrSparseVec::new(11);
    atomically(|t| {
        storage.write(t, 1, VertexAnchor::Node(1))?;
        storage.write(t, 2, VertexAnchor::Curve(2))?;
        storage.write(t, 3, VertexAnchor::Surface(3))?;
        storage.write(t, 4, VertexAnchor::Body(4))?;
        storage.write(t, 5, VertexAnchor::Body(5))?;
        storage.write(t, 6, VertexAnchor::Node(10))?;
        storage.write(t, 7, VertexAnchor::Curve(9))?;
        storage.write(t, 8, VertexAnchor::Surface(8))?;
        storage.write(t, 9, VertexAnchor::Body(7))?;
        storage.write(t, 10, VertexAnchor::Body(6))?;
        Ok(())
    });

    atomically(|t| {
        assert!(storage.merge(t, 3, 4, 3).is_ok());
        assert!(
            storage
                .read(t, 3)?
                .is_some_and(|v| v == VertexAnchor::Surface(3))
        );
        assert!(storage.merge(t, 2, 3, 2).is_ok());
        assert!(
            storage
                .read(t, 2)?
                .is_some_and(|v| v == VertexAnchor::Curve(2))
        );
        assert!(storage.merge(t, 1, 2, 1).is_ok());
        assert!(
            storage
                .read(t, 1)?
                .is_some_and(|v| v == VertexAnchor::Node(1))
        );
        assert!(storage.merge(t, 5, 1, 5).is_ok());
        assert!(
            storage
                .read(t, 5)?
                .is_some_and(|v| v == VertexAnchor::Node(1))
        );
        Ok(())
    });

    atomically(|t| {
        assert!(storage.merge(t, 6, 10, 6).is_ok());
        assert!(
            storage
                .read(t, 6)?
                .is_some_and(|v| v == VertexAnchor::Node(10))
        );
        assert!(storage.merge(t, 8, 8, 9).is_ok());
        assert!(
            storage
                .read(t, 8)?
                .is_some_and(|v| v == VertexAnchor::Surface(8))
        );
        assert!(storage.merge(t, 7, 7, 8).is_ok());
        assert!(
            storage
                .read(t, 7)?
                .is_some_and(|v| v == VertexAnchor::Curve(9))
        );
        assert!(storage.merge(t, 6, 6, 7).is_ok());
        assert!(
            storage
                .read(t, 6)?
                .is_some_and(|v| v == VertexAnchor::Node(10))
        );
        Ok(())
    });
}

#[test]
fn merge_edge_eq_dim() {
    let storage: AttrSparseVec<EdgeAnchor> = AttrSparseVec::new(13);
    atomically(|t| {
        storage.write(t, 4, EdgeAnchor::Curve(1))?;
        storage.write(t, 5, EdgeAnchor::Curve(1))?;
        storage.write(t, 6, EdgeAnchor::Curve(2))?;
        storage.write(t, 7, EdgeAnchor::Surface(3))?;
        storage.write(t, 8, EdgeAnchor::Surface(3))?;
        storage.write(t, 9, EdgeAnchor::Surface(4))?;
        storage.write(t, 10, EdgeAnchor::Body(5))?;
        storage.write(t, 11, EdgeAnchor::Body(6))?;
        storage.write(t, 12, EdgeAnchor::Body(6))?;
        Ok(())
    });

    atomically(|t| {
        // Curve merges
        assert!(storage.merge(t, 4, 4, 5).is_ok());
        assert!(storage.read(t, 4)?.is_some());
        assert!(storage.read(t, 5)?.is_none());
        assert!(storage.merge(t, 6, 4, 6).is_err());
        assert!(storage.read(t, 6)?.is_some());
        // Surface merges
        assert!(storage.merge(t, 7, 7, 8).is_ok());
        assert!(storage.read(t, 7)?.is_some());
        assert!(storage.read(t, 8)?.is_none());
        assert!(storage.merge(t, 7, 7, 9).is_err());
        assert!(storage.read(t, 9)?.is_some());
        // Body merges
        assert!(storage.merge(t, 11, 12, 11).is_ok());
        assert!(storage.read(t, 11)?.is_some());
        assert!(storage.read(t, 12)?.is_none());
        assert!(storage.merge(t, 10, 10, 11).is_err());
        assert!(storage.read(t, 11)?.is_some());
        Ok(())
    });
}

#[test]
fn merge_edge_diff_dim() {
    let storage: AttrSparseVec<EdgeAnchor> = AttrSparseVec::new(11);
    atomically(|t| {
        storage.write(t, 2, EdgeAnchor::Curve(2))?;
        storage.write(t, 3, EdgeAnchor::Surface(3))?;
        storage.write(t, 4, EdgeAnchor::Body(4))?;
        storage.write(t, 5, EdgeAnchor::Body(5))?;
        storage.write(t, 7, EdgeAnchor::Curve(9))?;
        storage.write(t, 8, EdgeAnchor::Surface(8))?;
        storage.write(t, 9, EdgeAnchor::Body(7))?;
        storage.write(t, 10, EdgeAnchor::Body(6))?;
        Ok(())
    });

    atomically(|t| {
        assert!(storage.merge(t, 3, 4, 3).is_ok());
        assert!(
            storage
                .read(t, 3)?
                .is_some_and(|v| v == EdgeAnchor::Surface(3))
        );
        assert!(storage.merge(t, 2, 3, 2).is_ok());
        assert!(
            storage
                .read(t, 2)?
                .is_some_and(|v| v == EdgeAnchor::Curve(2))
        );
        assert!(storage.merge(t, 5, 2, 5).is_ok());
        assert!(
            storage
                .read(t, 5)?
                .is_some_and(|v| v == EdgeAnchor::Curve(2))
        );
        Ok(())
    });

    atomically(|t| {
        assert!(storage.merge(t, 8, 8, 9).is_ok());
        assert!(
            storage
                .read(t, 8)?
                .is_some_and(|v| v == EdgeAnchor::Surface(8))
        );
        assert!(storage.merge(t, 7, 7, 8).is_ok());
        assert!(
            storage
                .read(t, 7)?
                .is_some_and(|v| v == EdgeAnchor::Curve(9))
        );
        assert!(storage.merge(t, 10, 10, 7).is_ok());
        assert!(
            storage
                .read(t, 10)?
                .is_some_and(|v| v == EdgeAnchor::Curve(9))
        );
        Ok(())
    });
}

#[test]
fn merge_face_eq_dim() {
    let storage: AttrSparseVec<FaceAnchor> = AttrSparseVec::new(13);
    atomically(|t| {
        storage.write(t, 7, FaceAnchor::Surface(3))?;
        storage.write(t, 8, FaceAnchor::Surface(3))?;
        storage.write(t, 9, FaceAnchor::Surface(4))?;
        storage.write(t, 10, FaceAnchor::Body(5))?;
        storage.write(t, 11, FaceAnchor::Body(6))?;
        storage.write(t, 12, FaceAnchor::Body(6))?;
        Ok(())
    });

    atomically(|t| {
        // Surface merges
        assert!(storage.merge(t, 7, 7, 8).is_ok());
        assert!(storage.read(t, 7)?.is_some());
        assert!(storage.read(t, 8)?.is_none());
        assert!(storage.merge(t, 7, 7, 9).is_err());
        assert!(storage.read(t, 9)?.is_some());
        // Body merges
        assert!(storage.merge(t, 11, 12, 11).is_ok());
        assert!(storage.read(t, 11)?.is_some());
        assert!(storage.read(t, 12)?.is_none());
        assert!(storage.merge(t, 10, 10, 11).is_err());
        assert!(storage.read(t, 11)?.is_some());
        Ok(())
    });
}

#[test]
fn merge_face_diff_dim() {
    let storage: AttrSparseVec<FaceAnchor> = AttrSparseVec::new(11);
    atomically(|t| {
        storage.write(t, 3, FaceAnchor::Surface(3))?;
        storage.write(t, 4, FaceAnchor::Body(4))?;
        storage.write(t, 5, FaceAnchor::Body(5))?;
        storage.write(t, 8, FaceAnchor::Surface(8))?;
        storage.write(t, 9, FaceAnchor::Body(7))?;
        storage.write(t, 10, FaceAnchor::Body(6))?;
        Ok(())
    });

    atomically(|t| {
        assert!(storage.merge(t, 3, 4, 3).is_ok());
        assert!(
            storage
                .read(t, 3)?
                .is_some_and(|v| v == FaceAnchor::Surface(3))
        );
        assert!(storage.merge(t, 5, 3, 5).is_ok());
        assert!(
            storage
                .read(t, 5)?
                .is_some_and(|v| v == FaceAnchor::Surface(3))
        );
        Ok(())
    });

    atomically(|t| {
        assert!(storage.merge(t, 8, 8, 9).is_ok());
        assert!(
            storage
                .read(t, 8)?
                .is_some_and(|v| v == FaceAnchor::Surface(8))
        );
        assert!(storage.merge(t, 10, 10, 8).is_ok());
        assert!(
            storage
                .read(t, 10)?
                .is_some_and(|v| v == FaceAnchor::Surface(8))
        );
        Ok(())
    });
}

#[test]
fn split_anchors() {
    let storage: AttrSparseVec<VertexAnchor> = AttrSparseVec::new(13);
    atomically(|t| {
        storage.write(t, 1, VertexAnchor::Node(1))?;
        storage.write(t, 4, VertexAnchor::Curve(1))?;
        storage.write(t, 7, VertexAnchor::Surface(3))?;
        storage.write(t, 10, VertexAnchor::Body(5))?;
        Ok(())
    });

    atomically(|t| {
        // Node split
        assert!(storage.split(t, 2, 3, 1).is_ok());
        assert!(storage.read(t, 1)?.is_none());
        assert!(storage.read(t, 2)?.is_some());
        assert!(storage.read(t, 3)?.is_some());
        // Curve split
        assert!(storage.split(t, 5, 6, 4).is_ok());
        assert!(storage.read(t, 4)?.is_none());
        assert!(storage.read(t, 5)?.is_some());
        assert!(storage.read(t, 6)?.is_some());
        // Surface split
        assert!(storage.split(t, 9, 8, 7).is_ok());
        assert!(storage.read(t, 7)?.is_none());
        assert!(storage.read(t, 8)?.is_some());
        assert!(storage.read(t, 9)?.is_some());
        // Body split
        assert!(storage.split(t, 11, 12, 10).is_ok());
        assert!(storage.read(t, 10)?.is_none());
        assert!(storage.read(t, 11)?.is_some());
        assert!(storage.read(t, 12)?.is_some());
        Ok(())
    });
}

// --- capture and classification

#[test]
fn capture_example() {
    // how likely is this to break?
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/shape.vtk");
    let map = capture_geometry(path, [1.0; 2], Clip::Right).unwrap();

    // there should be 13 nodes, 13 curves, and a single surface
    assert_eq!(
        map.iter_vertices()
            .filter_map(|v| map.force_read_attribute::<VertexAnchor>(v))
            .count(),
        13
    );

    classify_capture(&map).unwrap();

    let mut set = HashSet::new();
    map.iter_edges()
        .filter_map(|e| map.force_read_attribute::<EdgeAnchor>(e))
        .for_each(|a| {
            set.insert(a);
        });
    assert_eq!(
        set.iter()
            .filter(|a| matches!(*a, EdgeAnchor::Curve(_)))
            .count(),
        13
    );
    assert!(
        set.iter()
            .filter(|a| matches!(*a, EdgeAnchor::Surface(_)))
            .all(|a| match a {
                EdgeAnchor::Surface(id) => *id == 0,
                _ => unreachable!(),
            }),
    );
}

#[test]
fn classify_without_anchors() {
    let map: CMap2<_> = CMapBuilder::<2, f64>::from_n_darts(10).build().unwrap();
    assert!(
        classify_capture(&map)
            .is_err_and(|e| matches!(e, ClassificationError::MissingAttribute(_)))
    );

    let map: CMap2<_> = CMapBuilder::<2, f64>::from_n_darts(10)
        .add_attribute::<VertexAnchor>()
        .build()
        .unwrap();
    assert!(
        classify_capture(&map)
            .is_err_and(|e| matches!(e, ClassificationError::MissingAttribute(_)))
    );

    let map: CMap2<_> = CMapBuilder::<2, f64>::from_n_darts(10)
        .add_attribute::<VertexAnchor>()
        .add_attribute::<EdgeAnchor>()
        .build()
        .unwrap();
    assert!(
        classify_capture(&map)
            .is_err_and(|e| matches!(e, ClassificationError::MissingAttribute(_)))
    );
}

#[test]
fn classify_without_anchored_vertex_values() {
    // classifying a map with no anchored vertices values should result in all
    // cells being anchored to surfaces
    let mut map: CMap2<_> = CMapBuilder::<2, f64>::unit_grid(4)
        .add_attribute::<VertexAnchor>()
        .add_attribute::<EdgeAnchor>()
        .add_attribute::<FaceAnchor>()
        .build()
        .unwrap();
    // disjoint surfaces should have different IDs
    let d1 = map.add_free_darts(4);
    let (d2, d3, d4) = (d1 + 1, d1 + 2, d1 + 3);
    let _ = map.force_link::<1>(d1, d2);
    let _ = map.force_link::<1>(d2, d3);
    let _ = map.force_link::<1>(d3, d4);
    let _ = map.force_link::<1>(d4, d1);

    assert_eq!(classify_capture(&map), Ok(()));

    // all cells are classified on surfaces except boundaries
    let (on_boundary, off_boundary): (Vec<_>, Vec<_>) = map.iter_vertices().partition(|v| {
        map.orbit(OrbitPolicy::Vertex, *v)
            .any(|dd| map.beta::<2>(dd) == NULL_DART_ID)
    });
    assert!(on_boundary.into_iter().all(|v| matches!(
        map.force_read_attribute::<VertexAnchor>(v),
        Some(VertexAnchor::Curve(_))
    )));
    assert!(off_boundary.into_iter().all(|v| matches!(
        map.force_read_attribute::<VertexAnchor>(v),
        Some(VertexAnchor::Surface(_))
    )));

    let (on_boundary, off_boundary): (Vec<_>, Vec<_>) = map
        .iter_edges()
        .partition(|e| map.beta::<2>(*e) == NULL_DART_ID);
    assert!(on_boundary.into_iter().all(|e| matches!(
        map.force_read_attribute::<EdgeAnchor>(e),
        Some(EdgeAnchor::Curve(_))
    )));
    assert!(off_boundary.into_iter().all(|e| matches!(
        map.force_read_attribute::<EdgeAnchor>(e),
        Some(EdgeAnchor::Surface(_))
    )));

    assert!(map.iter_faces().all(|f| matches!(
        map.force_read_attribute::<FaceAnchor>(f),
        Some(FaceAnchor::Surface(_))
    )));

    // there are two disjoint boundaries => curves
    assert!(matches!(
        map.force_read_attribute::<VertexAnchor>(1),
        Some(VertexAnchor::Curve(_))
    ));
    assert!(matches!(
        map.force_read_attribute::<VertexAnchor>(d1),
        Some(VertexAnchor::Curve(_))
    ));
    assert_ne!(
        map.force_read_attribute::<VertexAnchor>(1),
        map.force_read_attribute::<VertexAnchor>(d1)
    );
}

#[test]
fn classify_with_anchored_vertex_values() {
    // classifying a map with no anchored vertices values should result in all
    // cells being anchored to surfaces
    let map: CMap2<_> = CMapBuilder::<2, f64>::unit_grid(2)
        .add_attribute::<VertexAnchor>()
        .add_attribute::<EdgeAnchor>()
        .add_attribute::<FaceAnchor>()
        .build()
        .unwrap();
    map.force_write_attribute(1, VertexAnchor::Node(1));
    map.force_write_attribute(6, VertexAnchor::Node(2));
    map.force_write_attribute(12, VertexAnchor::Node(3));
    map.force_write_attribute(15, VertexAnchor::Node(4));

    assert_eq!(classify_capture(&map), Ok(()));

    // all cells are classified on surfaces except boundaries
    let (on_boundary, off_boundary): (Vec<_>, Vec<_>) = map.iter_vertices().partition(|v| {
        map.orbit(OrbitPolicy::Vertex, *v)
            .any(|dd| map.beta::<2>(dd) == NULL_DART_ID)
    });
    assert!(on_boundary.into_iter().all(|v| matches!(
        map.force_read_attribute::<VertexAnchor>(v),
        Some(VertexAnchor::Curve(_)) | Some(VertexAnchor::Node(_)) // we have 4 nodes in there
    )));
    assert!(off_boundary.into_iter().all(|v| matches!(
        map.force_read_attribute::<VertexAnchor>(v),
        Some(VertexAnchor::Surface(_))
    )));

    let (on_boundary, off_boundary): (Vec<_>, Vec<_>) = map
        .iter_edges()
        .partition(|e| map.beta::<2>(*e) == NULL_DART_ID);
    assert!(on_boundary.into_iter().all(|e| matches!(
        map.force_read_attribute::<EdgeAnchor>(e),
        Some(EdgeAnchor::Curve(_))
    )));
    assert!(off_boundary.into_iter().all(|e| matches!(
        map.force_read_attribute::<EdgeAnchor>(e),
        Some(EdgeAnchor::Surface(_))
    )));

    assert!(map.iter_faces().all(|f| matches!(
        map.force_read_attribute::<FaceAnchor>(f),
        Some(FaceAnchor::Surface(_))
    )));
}

// --- edge swap

#[test]
fn swap_edge_errs() {
    let map = CMapBuilder::<2, f64>::unit_triangles(1).build().unwrap();

    assert!(
        atomically_with_err(|t| swap_edge(t, &map, 0)).is_err_and(|e| e == EdgeSwapError::NullEdge)
    );
    assert!(
        atomically_with_err(|t| swap_edge(t, &map, 1))
            .is_err_and(|e| e == EdgeSwapError::IncompleteEdge)
    );

    let map = CMapBuilder::<2, f64>::unit_grid(2).build().unwrap();

    assert!(
        atomically_with_err(|t| swap_edge(t, &map, 2))
            .is_err_and(|e| e == EdgeSwapError::BadTopology)
    );
}

#[test]
fn swap_edge_seq() {
    let map = CMapBuilder::<2, f64>::unit_triangles(1).build().unwrap();

    // before
    //
    // +---+
    // |\  |
    // | \ |
    // |  \|
    // +---+

    let o1: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 1).collect();
    assert_eq!(&o1, &[1, 2, 3]);
    let o6: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 6).collect();
    assert_eq!(&o6, &[6, 4, 5]);

    assert!(atomically_with_err(|t| swap_edge(t, &map, 2)).is_ok());

    // after
    //
    // +---+
    // |  /|
    // | / |
    // |/  |
    // +---+

    let o1: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 1).collect();
    assert_eq!(&o1, &[1, 5, 4]);
    let o6: Vec<_> = map.orbit(OrbitPolicy::FaceLinear, 6).collect();
    assert_eq!(&o6, &[6, 3, 2]);
}
