use honeycomb_core::cmap::{CMapBuilder, GridDescriptor, Orbit2, OrbitPolicy};
use honeycomb_core::geometry::Vertex2;
use vtkio::Vtk;

use crate::grisubal::model::{Boundary, Geometry2, GeometryVertex};
use crate::grisubal::routines::{
    compute_intersection_ids, generate_edge_data, generate_intersection_data,
    group_intersections_per_edge, insert_edges_in_map, insert_intersections,
};

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
    let geometry: Geometry2<f32> = Geometry2::try_from(vtk).unwrap();
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
    let mut cmap = CMapBuilder::from(
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
        generate_intersection_data(&cmap, &geometry, [2, 2], [1.0, 1.0], Vertex2::default());

    assert_eq!(intersection_metadata.len(), 4);
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

    let n_intersec = intersection_metadata.len();
    let (edge_intersec, dart_slices) =
        group_intersections_per_edge(&mut cmap, intersection_metadata);
    let intersection_darts = compute_intersection_ids(n_intersec, &edge_intersec, &dart_slices);
    insert_intersections(&mut cmap, &edge_intersec, &dart_slices);

    assert_eq!(n_intersec, 4);
    // check new vertices at intersection
    assert_eq!(
        cmap.force_read_vertex(cmap.vertex_id(cmap.beta::<1>(2))),
        Some(Vertex2(1.0, 0.5))
    );
    assert_eq!(
        cmap.force_read_vertex(cmap.vertex_id(cmap.beta::<1>(7))),
        Some(Vertex2(1.5, 1.0))
    );
    assert_eq!(
        cmap.force_read_vertex(cmap.vertex_id(cmap.beta::<1>(10))),
        Some(Vertex2(1.0, 1.5))
    );
    assert_eq!(
        cmap.force_read_vertex(cmap.vertex_id(cmap.beta::<1>(3))),
        Some(Vertex2(0.5, 1.0))
    );

    let mut edges = generate_edge_data(&cmap, &geometry, &segments, &intersection_darts);

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

    let faces: Vec<_> = cmap.iter_faces().collect();
    assert_eq!(faces.len(), 8);
    // bottom left
    assert!(faces.contains(&1));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 1).count(), 6);
    assert!(faces.contains(&3));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 3).count(), 4);
    // bottom right
    assert!(faces.contains(&5));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 5).count(), 6);
    assert!(faces.contains(&8));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 8).count(), 4);
    // top right
    assert!(faces.contains(&9));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 9).count(), 6);
    assert!(faces.contains(&10));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 10).count(), 4);
    // top left
    assert!(faces.contains(&14));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 14).count(), 6);
    assert!(faces.contains(&13));
    assert_eq!(Orbit2::new(&cmap, OrbitPolicy::Face, 13).count(), 4);
}

#[allow(clippy::too_many_lines)]
#[test]
fn corner_intersection() {
    use num_traits::Float;

    let mut cmap = CMapBuilder::from(
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
        generate_intersection_data(&cmap, &geometry, [2, 2], [1.0, 1.0], Vertex2::default());

    // because we intersec a corner, some entries were preallocated but not needed.
    // entries were initialized with (0, Nan), so they're easy to filter
    assert_eq!(
        intersection_metadata
            .iter()
            .filter(|(_, t)| !t.is_nan())
            .count(),
        2
    );
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

    let n_intersec = intersection_metadata.len();
    let (edge_intersec, dart_slices) =
        group_intersections_per_edge(&mut cmap, intersection_metadata);
    let intersection_darts = compute_intersection_ids(n_intersec, &edge_intersec, &dart_slices);
    insert_intersections(&mut cmap, &edge_intersec, &dart_slices);

    let mut edges = generate_edge_data(&cmap, &geometry, &segments, &intersection_darts);

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

    let faces = cmap.iter_faces();
    assert_eq!(faces.count(), 7);
    let edges = cmap.iter_edges();
    assert_eq!(edges.count(), 20);

    let face1_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 1)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face1_vertices.len(), 6);
    assert!(face1_vertices.contains(&Vertex2(0.0, 0.0)));
    assert!(face1_vertices.contains(&Vertex2(1.0, 0.0)));
    assert!(face1_vertices.contains(&Vertex2(1.0, 0.5)));
    assert!(face1_vertices.contains(&Vertex2(0.5, 0.5)));
    assert!(face1_vertices.contains(&Vertex2(1.0, 1.0)));
    assert!(face1_vertices.contains(&Vertex2(0.0, 1.0)));

    let face9_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 9)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face9_vertices.len(), 4);
    assert!(face9_vertices.contains(&Vertex2(0.0, 1.0)));
    assert!(face9_vertices.contains(&Vertex2(1.0, 1.0)));
    assert!(face9_vertices.contains(&Vertex2(1.0, 2.0)));
    assert!(face9_vertices.contains(&Vertex2(0.0, 2.0)));

    let face13_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 13)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face13_vertices.len(), 3);
    assert!(face13_vertices.contains(&Vertex2(1.0, 1.0)));
    assert!(face13_vertices.contains(&Vertex2(1.5, 1.0)));
    assert!(face13_vertices.contains(&Vertex2(1.5, 1.5)));
}

#[allow(clippy::too_many_lines)]
#[test]
pub fn successive_straight_intersections() {
    let mut cmap = CMapBuilder::from(
        GridDescriptor::default()
            .len_per_cell([1.0; 3])
            .n_cells([3; 3]),
    )
    .add_attribute::<Boundary>()
    .build()
    .unwrap();

    // square where each corner belong to non-neighboring cells
    let geometry = Geometry2 {
        vertices: vec![
            Vertex2(0.5, 0.5),
            Vertex2(2.5, 0.5),
            Vertex2(2.5, 2.5),
            Vertex2(0.5, 2.5),
        ],
        segments: vec![(0, 1), (1, 2), (2, 3), (3, 0)],
        poi: vec![0, 1, 2, 3],
    };

    let (segments, intersection_metadata) =
        generate_intersection_data(&cmap, &geometry, [3, 3], [1.0, 1.0], Vertex2::default());

    let n_intersec = intersection_metadata.len();
    let (edge_intersec, dart_slices) =
        group_intersections_per_edge(&mut cmap, intersection_metadata);
    let intersection_darts = compute_intersection_ids(n_intersec, &edge_intersec, &dart_slices);
    insert_intersections(&mut cmap, &edge_intersec, &dart_slices);

    let edges = generate_edge_data(&cmap, &geometry, &segments, &intersection_darts);

    assert_eq!(edges.len(), 8);
    assert_eq!(
        edges
            .iter()
            .filter(|edge| !edge.intermediates.is_empty())
            .count(),
        4
    );

    insert_edges_in_map(&mut cmap, &edges);

    // bottom row

    let face1_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 1)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face1_vertices.len(), 6);
    assert!(face1_vertices.contains(&Vertex2(0.0, 0.0)));
    assert!(face1_vertices.contains(&Vertex2(1.0, 0.0)));
    assert!(face1_vertices.contains(&Vertex2(1.0, 0.5)));
    assert!(face1_vertices.contains(&Vertex2(0.5, 0.5)));
    assert!(face1_vertices.contains(&Vertex2(0.5, 1.0)));
    assert!(face1_vertices.contains(&Vertex2(0.0, 1.0)));

    let face3_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 3)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face3_vertices.len(), 4);
    assert!(face3_vertices.contains(&Vertex2(0.5, 0.5)));
    assert!(face3_vertices.contains(&Vertex2(1.0, 0.5)));
    assert!(face3_vertices.contains(&Vertex2(1.0, 1.0)));
    assert!(face3_vertices.contains(&Vertex2(0.5, 1.0)));

    let face5_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 5)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face5_vertices.len(), 4);
    assert!(face5_vertices.contains(&Vertex2(1.0, 0.0)));
    assert!(face5_vertices.contains(&Vertex2(2.0, 0.0)));
    assert!(face5_vertices.contains(&Vertex2(2.0, 0.5)));
    assert!(face5_vertices.contains(&Vertex2(1.0, 0.5)));

    let face7_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 7)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face7_vertices.len(), 4);
    assert!(face7_vertices.contains(&Vertex2(1.0, 1.0)));
    assert!(face7_vertices.contains(&Vertex2(2.0, 1.0)));
    assert!(face7_vertices.contains(&Vertex2(2.0, 0.5)));
    assert!(face7_vertices.contains(&Vertex2(1.0, 0.5)));

    let face9_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 9)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face9_vertices.len(), 6);
    assert!(face9_vertices.contains(&Vertex2(2.0, 0.0)));
    assert!(face9_vertices.contains(&Vertex2(3.0, 0.0)));
    assert!(face9_vertices.contains(&Vertex2(3.0, 1.0)));
    assert!(face9_vertices.contains(&Vertex2(2.5, 1.0)));
    assert!(face9_vertices.contains(&Vertex2(2.5, 0.5)));
    assert!(face9_vertices.contains(&Vertex2(2.0, 0.5)));

    let face12_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 12)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face12_vertices.len(), 4);
    assert!(face12_vertices.contains(&Vertex2(2.0, 0.5)));
    assert!(face12_vertices.contains(&Vertex2(2.5, 0.5)));
    assert!(face12_vertices.contains(&Vertex2(2.5, 1.0)));
    assert!(face12_vertices.contains(&Vertex2(2.0, 1.0)));

    // middle row

    let face13_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 13)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face13_vertices.len(), 4);
    assert!(face13_vertices.contains(&Vertex2(0.0, 2.0)));
    assert!(face13_vertices.contains(&Vertex2(0.0, 1.0)));
    assert!(face13_vertices.contains(&Vertex2(0.5, 1.0)));
    assert!(face13_vertices.contains(&Vertex2(0.5, 2.0)));

    let face14_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 14)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face14_vertices.len(), 4);
    assert!(face14_vertices.contains(&Vertex2(0.5, 1.0)));
    assert!(face14_vertices.contains(&Vertex2(0.5, 2.0)));
    assert!(face14_vertices.contains(&Vertex2(1.0, 2.0)));
    assert!(face14_vertices.contains(&Vertex2(1.0, 1.0)));

    let face17_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 17)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face17_vertices.len(), 4);
    assert!(face17_vertices.contains(&Vertex2(1.0, 1.0)));
    assert!(face17_vertices.contains(&Vertex2(1.0, 2.0)));
    assert!(face17_vertices.contains(&Vertex2(2.0, 2.0)));
    assert!(face17_vertices.contains(&Vertex2(2.0, 1.0)));

    let face21_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 21)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face21_vertices.len(), 4);
    assert!(face21_vertices.contains(&Vertex2(2.0, 1.0)));
    assert!(face21_vertices.contains(&Vertex2(2.5, 1.0)));
    assert!(face21_vertices.contains(&Vertex2(2.5, 2.0)));
    assert!(face21_vertices.contains(&Vertex2(2.0, 2.0)));

    let face22_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 22)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face22_vertices.len(), 4);
    assert!(face22_vertices.contains(&Vertex2(2.5, 1.0)));
    assert!(face22_vertices.contains(&Vertex2(2.5, 2.0)));
    assert!(face22_vertices.contains(&Vertex2(3.0, 2.0)));
    assert!(face22_vertices.contains(&Vertex2(3.0, 1.0)));

    // top row

    let face25_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 25)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face25_vertices.len(), 6);
    assert!(face25_vertices.contains(&Vertex2(0.0, 2.0)));
    assert!(face25_vertices.contains(&Vertex2(0.5, 2.0)));
    assert!(face25_vertices.contains(&Vertex2(0.5, 2.5)));
    assert!(face25_vertices.contains(&Vertex2(1.0, 2.5)));
    assert!(face25_vertices.contains(&Vertex2(1.0, 3.0)));
    assert!(face25_vertices.contains(&Vertex2(0.0, 3.0)));

    let face26_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 26)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face26_vertices.len(), 4);
    assert!(face26_vertices.contains(&Vertex2(0.5, 2.0)));
    assert!(face26_vertices.contains(&Vertex2(1.0, 2.0)));
    assert!(face26_vertices.contains(&Vertex2(1.0, 2.5)));
    assert!(face26_vertices.contains(&Vertex2(0.5, 2.5)));

    let face29_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 29)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face29_vertices.len(), 4);
    assert!(face29_vertices.contains(&Vertex2(1.0, 2.0)));
    assert!(face29_vertices.contains(&Vertex2(2.0, 2.0)));
    assert!(face29_vertices.contains(&Vertex2(2.0, 2.5)));
    assert!(face29_vertices.contains(&Vertex2(1.0, 2.5)));

    let face31_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 31)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face31_vertices.len(), 4);
    assert!(face31_vertices.contains(&Vertex2(2.0, 2.5)));
    assert!(face31_vertices.contains(&Vertex2(1.0, 2.5)));
    assert!(face31_vertices.contains(&Vertex2(1.0, 3.0)));
    assert!(face31_vertices.contains(&Vertex2(2.0, 3.0)));

    let face33_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 33)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face33_vertices.len(), 4);
    assert!(face33_vertices.contains(&Vertex2(2.0, 2.0)));
    assert!(face33_vertices.contains(&Vertex2(2.0, 2.5)));
    assert!(face33_vertices.contains(&Vertex2(2.5, 2.5)));
    assert!(face33_vertices.contains(&Vertex2(2.5, 2.0)));

    let face34_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 34)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face34_vertices.len(), 6);
    assert!(face34_vertices.contains(&Vertex2(2.5, 2.0)));
    assert!(face34_vertices.contains(&Vertex2(3.0, 2.0)));
    assert!(face34_vertices.contains(&Vertex2(3.0, 3.0)));
    assert!(face34_vertices.contains(&Vertex2(2.0, 3.0)));
    assert!(face34_vertices.contains(&Vertex2(2.0, 2.5)));
    assert!(face34_vertices.contains(&Vertex2(2.5, 2.5)));
}

#[allow(clippy::too_many_lines)]
#[test]
pub fn successive_diag_intersections() {
    let mut cmap = CMapBuilder::from(
        GridDescriptor::default()
            .len_per_cell([1.0; 3])
            .n_cells([3; 3]),
    )
    .add_attribute::<Boundary>()
    .build()
    .unwrap();

    // square where each corner belong to non-neighboring cells
    let geometry = Geometry2 {
        vertices: vec![
            Vertex2(1.33, 0.5),
            Vertex2(1.66, 0.5),
            Vertex2(2.5, 1.33),
            Vertex2(2.5, 1.66),
            Vertex2(1.66, 2.5),
            Vertex2(1.33, 2.5),
            Vertex2(0.5, 1.66),
            Vertex2(0.5, 1.33),
        ],
        segments: vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 0),
        ],
        poi: vec![0, 1, 2, 3, 4, 5, 6, 7],
    };

    let (segments, intersection_metadata) =
        generate_intersection_data(&cmap, &geometry, [3, 3], [1.0, 1.0], Vertex2::default());

    let n_intersec = intersection_metadata.len();
    let (edge_intersec, dart_slices) =
        group_intersections_per_edge(&mut cmap, intersection_metadata);
    let intersection_darts = compute_intersection_ids(n_intersec, &edge_intersec, &dart_slices);
    insert_intersections(&mut cmap, &edge_intersec, &dart_slices);

    let edges = generate_edge_data(&cmap, &geometry, &segments, &intersection_darts);

    assert_eq!(edges.len(), 8);
    assert_eq!(
        edges
            .iter()
            .filter(|edge| !edge.intermediates.is_empty())
            .count(),
        4
    );

    insert_edges_in_map(&mut cmap, &edges);

    // bottom row

    let face1_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 1)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face1_vertices.len(), 5);

    let face3_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 3)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face3_vertices.len(), 3);

    let face5_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 5)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face5_vertices.len(), 6);

    let face7_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 7)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face7_vertices.len(), 6);

    let face9_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 9)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face9_vertices.len(), 5);

    let face12_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 12)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face12_vertices.len(), 3);

    // middle row

    let face13_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 13)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face13_vertices.len(), 6);

    let face14_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 14)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face14_vertices.len(), 6);

    let face17_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 17)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face17_vertices.len(), 4);

    let face21_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 21)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face21_vertices.len(), 6);

    let face22_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 22)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face22_vertices.len(), 6);

    // top row

    let face25_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 25)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face25_vertices.len(), 5);

    let face26_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 26)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face26_vertices.len(), 3);

    let face29_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 29)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face29_vertices.len(), 6);

    let face31_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 31)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face31_vertices.len(), 6);

    let face33_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 33)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face33_vertices.len(), 3);

    let face34_vertices: Vec<Vertex2<f64>> = Orbit2::new(&cmap, OrbitPolicy::Face, 34)
        .map(|d| {
            cmap.force_read_vertex(cmap.vertex_id(d))
                .expect("E: unreachable")
        })
        .collect();
    assert_eq!(face34_vertices.len(), 5);
}
