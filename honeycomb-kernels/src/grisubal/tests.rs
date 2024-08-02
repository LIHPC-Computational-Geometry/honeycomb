// ------ IMPORTS

use honeycomb_core::{CMapBuilder, GridDescriptor, Orbit2, OrbitPolicy, Vertex2};
use vtkio::Vtk;

use crate::{
    grisubal::kernel::{
        generate_edge_data, generate_intersection_data, insert_edges_in_map, insert_intersections,
    },
    Boundary, Geometry2, GeometryVertex,
};

// ------ CONTENT

// --- geometry building

#[cfg(test)]
const VTK_ASCII: &[u8] = b"
# vtk DataFile Version 2.0
cmap
ASCII

DATASET UNSTRUCTURED_GRID
POINTS 9 float
0 0 0  1 0 0  1 1 0
0 1 0  2 0 0  2 1 0
2 2 0  1 3 0  0 2 0

CELLS 17 54
1 0
1 4
1 6
1 7
1 8
2 0 1
2 3 0
2 1 4
2 4 5
2 5 6
2 6 7
2 7 8
2 8 3
4 0 1 2 3
3 1 4 5
3 1 5 2
6 3 2 5 6 7 8

CELL_TYPES 17
1
1
1
1
1
3
3
3
3
3
3
3
3
9
5
5
7


POINT_DATA 9

CELL_DATA 17
";

#[test]
fn build_valid_geometry() {
    // parse vtk
    let vtk = Vtk::parse_legacy_be(VTK_ASCII).unwrap();
    // build geometry
    let geometry: Geometry2<f32> = Geometry2::from(vtk);
    // check results; we're expecting:
    // - 9 vertices
    // - 8 segments making up the boundaries
    // - 5 points of interest out of the 9 vertices
    let Geometry2 {
        vertices,
        segments,
        poi,
    } = geometry;
    // vertices
    assert_eq!(vertices.len(), 9);
    assert!(vertices.contains(&Vertex2::from((0., 0.))));
    assert!(vertices.contains(&Vertex2::from((1., 0.))));
    assert!(vertices.contains(&Vertex2::from((2., 0.))));
    assert!(vertices.contains(&Vertex2::from((0., 1.))));
    assert!(vertices.contains(&Vertex2::from((1., 1.))));
    assert!(vertices.contains(&Vertex2::from((2., 1.))));
    assert!(vertices.contains(&Vertex2::from((0., 2.))));
    assert!(vertices.contains(&Vertex2::from((1., 3.))));
    assert!(vertices.contains(&Vertex2::from((2., 2.))));
    // segments
    assert_eq!(segments.len(), 8);
    assert!(segments.contains(&(0, 1)));
    assert!(segments.contains(&(3, 0)));
    assert!(segments.contains(&(1, 4)));
    assert!(segments.contains(&(4, 5)));
    assert!(segments.contains(&(5, 6)));
    assert!(segments.contains(&(6, 7)));
    assert!(segments.contains(&(7, 8)));
    assert!(segments.contains(&(8, 3)));
    // poi
    assert_eq!(poi.len(), 5);
    assert!(poi.contains(&0));
    assert!(poi.contains(&4));
    assert!(poi.contains(&6));
    assert!(poi.contains(&7));
    assert!(poi.contains(&8));
}

#[allow(clippy::too_many_lines)]
#[test]
fn regular_intersections() {
    let mut cmap = CMapBuilder::from_grid_descriptor(
        GridDescriptor::default()
            .len_per_cell([1.0; 3])
            .n_cells([2; 3]),
    )
    .add_attribute::<Boundary>()
    .build()
    .unwrap();

    // square with bottom left at (0.5,0.5) & top right at (1.5,1.5)
    let geometry = Geometry2 {
        vertices: vec![
            Vertex2(0.5, 0.5),
            Vertex2(1.5, 0.5),
            Vertex2(1.5, 1.5),
            Vertex2(0.5, 1.5),
        ],
        segments: vec![(0, 1), (1, 2), (2, 3), (3, 0)],
        poi: vec![0, 1, 2, 3],
    };

    let (segments, intersection_metadata) =
        generate_intersection_data(&mut cmap, &geometry, [2, 2], [1.0, 1.0]);

    assert_eq!(intersection_metadata.len(), 4);
    // FIXME: INDEX ACCESSES WON'T WORK IN PARALLEL
    assert_eq!(intersection_metadata[0], (2, 0.5));
    assert_eq!(intersection_metadata[1], (7, 0.5));
    assert_eq!(intersection_metadata[2], (16, 0.5));
    assert_eq!(intersection_metadata[3], (9, 0.5));
    // go through the segments
    assert!(segments.contains_key(&GeometryVertex::PoI(0)));
    assert_eq!(
        segments[&GeometryVertex::PoI(0)],
        GeometryVertex::Intersec(0)
    );
    assert!(segments.contains_key(&GeometryVertex::Intersec(0)));
    assert_eq!(
        segments[&GeometryVertex::Intersec(0)],
        GeometryVertex::PoI(1)
    );
    assert!(segments.contains_key(&GeometryVertex::PoI(1)));
    assert_eq!(
        segments[&GeometryVertex::PoI(1)],
        GeometryVertex::Intersec(1)
    );
    assert!(segments.contains_key(&GeometryVertex::Intersec(1)));
    assert_eq!(
        segments[&GeometryVertex::Intersec(1)],
        GeometryVertex::PoI(2)
    );
    assert!(segments.contains_key(&GeometryVertex::PoI(2)));
    assert_eq!(
        segments[&GeometryVertex::PoI(2)],
        GeometryVertex::Intersec(2)
    );
    assert!(segments.contains_key(&GeometryVertex::Intersec(2)));
    assert_eq!(
        segments[&GeometryVertex::Intersec(2)],
        GeometryVertex::PoI(3)
    );
    assert!(segments.contains_key(&GeometryVertex::PoI(3)));
    assert_eq!(
        segments[&GeometryVertex::PoI(3)],
        GeometryVertex::Intersec(3)
    );
    assert!(segments.contains_key(&GeometryVertex::Intersec(3)));
    assert_eq!(
        segments[&GeometryVertex::Intersec(3)],
        GeometryVertex::PoI(0)
    );

    let intersection_darts = insert_intersections(&mut cmap, intersection_metadata);

    assert_eq!(intersection_darts.len(), 4);
    // check new vertices at intersection
    assert_eq!(
        cmap.vertex(cmap.vertex_id(cmap.beta::<1>(2))),
        Ok(Vertex2(1.0, 0.5))
    );
    assert_eq!(
        cmap.vertex(cmap.vertex_id(cmap.beta::<1>(7))),
        Ok(Vertex2(1.5, 1.0))
    );
    assert_eq!(
        cmap.vertex(cmap.vertex_id(cmap.beta::<1>(10))),
        Ok(Vertex2(1.0, 1.5))
    );
    assert_eq!(
        cmap.vertex(cmap.vertex_id(cmap.beta::<1>(3))),
        Ok(Vertex2(0.5, 1.0))
    );

    let mut edges = generate_edge_data(&mut cmap, &geometry, &segments, &intersection_darts);

    assert_eq!(edges.len(), 4);
    edges.retain(|edge| !edge.intermediates.is_empty());
    assert_eq!(edges.len(), 4);

    insert_edges_in_map(&mut cmap, &edges);

    // we're expecting something like this
    // +-----+-----+
    // |     |     |
    // | 9   |   14|
    // |  +--+--+  |
    // |  |10|13|  |
    // |  |  |  |  |
    // +--+--+--+--+
    // |  |3 |8 |  |
    // |  |  |  |  |
    // |  +--+--+  |
    // | 1   |   5 |
    // |     |     |
    // +-----+-----+

    let faces = cmap.fetch_faces();
    assert_eq!(faces.identifiers.len(), 8);
    // bottom left
    assert!(faces.identifiers.contains(&1));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 1).count(), 6);
    assert!(faces.identifiers.contains(&3));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 3).count(), 4);
    // bottom right
    assert!(faces.identifiers.contains(&5));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 5).count(), 6);
    assert!(faces.identifiers.contains(&8));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 8).count(), 4);
    // top right
    assert!(faces.identifiers.contains(&9));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 9).count(), 6);
    assert!(faces.identifiers.contains(&10));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 10).count(), 4);
    // top left
    assert!(faces.identifiers.contains(&14));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 14).count(), 6);
    assert!(faces.identifiers.contains(&13));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 13).count(), 4);
}

#[test]
fn corner_intersection() {
    let mut cmap = CMapBuilder::from_grid_descriptor(
        GridDescriptor::default()
            .len_per_cell([1.0; 3])
            .n_cells([2; 3]),
    )
    .add_attribute::<Boundary>()
    .build()
    .unwrap();

    // square with bottom left at (0.5,0.5) & top right at (1.5,1.5)
    let geometry = Geometry2 {
        vertices: vec![Vertex2(0.5, 0.5), Vertex2(1.5, 0.5), Vertex2(1.5, 1.5)],
        segments: vec![(0, 1), (1, 2), (2, 0)],
        poi: vec![0, 1, 2],
    };

    let (segments, intersection_metadata) =
        generate_intersection_data(&mut cmap, &geometry, [2, 2], [1.0, 1.0]);

    assert_eq!(intersection_metadata.len(), 2);
    assert_eq!(intersection_metadata[0], (2, 0.5));
    assert_eq!(intersection_metadata[1], (7, 0.5));

    assert_eq!(segments.len(), 6);

    assert!(segments.contains_key(&GeometryVertex::PoI(0)));
    assert_eq!(
        segments[&GeometryVertex::PoI(0)],
        GeometryVertex::Intersec(0)
    );
    assert!(segments.contains_key(&GeometryVertex::Intersec(0)));
    assert_eq!(
        segments[&GeometryVertex::Intersec(0)],
        GeometryVertex::PoI(1)
    );
    assert!(segments.contains_key(&GeometryVertex::PoI(1)));
    assert_eq!(
        segments[&GeometryVertex::PoI(1)],
        GeometryVertex::Intersec(1)
    );
    assert!(segments.contains_key(&GeometryVertex::Intersec(1)));
    assert_eq!(
        segments[&GeometryVertex::Intersec(1)],
        GeometryVertex::PoI(2)
    );
    // !
    assert!(segments.contains_key(&GeometryVertex::PoI(2)));
    assert_eq!(
        segments[&GeometryVertex::PoI(2)],
        GeometryVertex::IntersecCorner(13)
    );
    // !
    assert!(segments.contains_key(&GeometryVertex::IntersecCorner(13)));
    assert_eq!(
        segments[&GeometryVertex::IntersecCorner(13)],
        GeometryVertex::PoI(0)
    );

    // same as the one of the `regular_intersections`, so we won't repeat the assertions
    let intersection_darts = insert_intersections(&mut cmap, intersection_metadata);

    let mut edges = generate_edge_data(&mut cmap, &geometry, &segments, &intersection_darts);

    assert_eq!(edges.len(), 3);
    edges.retain(|edge| !edge.intermediates.is_empty());
    assert_eq!(edges.len(), 3);

    insert_edges_in_map(&mut cmap, &edges);

    // we're expecting something like this
    // +-----+-----+
    // |     |     |
    // |     |     |
    // |     |  +  |
    // |     | /|  |
    // |     |/ |  |
    // +-----+--+--+
    // |    /|  |  |
    // |   / |  |  |
    // |  +--+--+  |
    // |     |     |
    // |     |     |
    // +-----+-----+

    let faces = cmap.fetch_faces();
    assert_eq!(faces.identifiers.len(), 7);
    let edges = cmap.fetch_edges();
    assert_eq!(edges.identifiers.len(), 20);

    let face1_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 1)
        .map(|d| cmap.vertex(cmap.vertex_id(d)).expect("E: unreachable"))
        .collect();
    assert_eq!(face1_vertices.len(), 6);
    assert!(face1_vertices.contains(&Vertex2(0.0, 0.0)));
    assert!(face1_vertices.contains(&Vertex2(1.0, 0.0)));
    assert!(face1_vertices.contains(&Vertex2(1.0, 0.5)));
    assert!(face1_vertices.contains(&Vertex2(0.5, 0.5)));
    assert!(face1_vertices.contains(&Vertex2(1.0, 1.0)));
    assert!(face1_vertices.contains(&Vertex2(0.0, 1.0)));

    let face9_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 9)
        .map(|d| cmap.vertex(cmap.vertex_id(d)).expect("E: unreachable"))
        .collect();
    assert_eq!(face9_vertices.len(), 4);
    assert!(face9_vertices.contains(&Vertex2(0.0, 1.0)));
    assert!(face9_vertices.contains(&Vertex2(1.0, 1.0)));
    assert!(face9_vertices.contains(&Vertex2(1.0, 2.0)));
    assert!(face9_vertices.contains(&Vertex2(0.0, 2.0)));

    let face13_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 13)
        .map(|d| cmap.vertex(cmap.vertex_id(d)).expect("E: unreachable"))
        .collect();
    assert_eq!(face13_vertices.len(), 3);
    assert!(face13_vertices.contains(&Vertex2(1.0, 1.0)));
    assert!(face13_vertices.contains(&Vertex2(1.5, 1.0)));
    assert!(face13_vertices.contains(&Vertex2(1.5, 1.5)));
}
