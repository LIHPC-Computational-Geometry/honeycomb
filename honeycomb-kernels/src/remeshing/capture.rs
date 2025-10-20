use std::collections::{HashSet, VecDeque};

use honeycomb_core::{
    cmap::{CMap2, DartIdType, NULL_DART_ID, OrbitPolicy},
    geometry::CoordsFloat,
};
use vtkio::Vtk;

use crate::utils::{CurveIdType, EdgeAnchor, FaceAnchor, VertexAnchor};
use crate::{
    grid_generation::GridBuilder,
    grisubal::{
        Clip, GrisubalError,
        model::{Boundary, Geometry2},
        routines::{
            clip_left, clip_right, compute_intersection_ids, compute_overlapping_grid,
            detect_orientation_issue, generate_edge_data, generate_intersection_data,
            group_intersections_per_edge, insert_edges_in_map, insert_intersections,
        },
    },
};

#[allow(clippy::missing_errors_doc)]
/// Capture the geometry specified as input using the `grisubal` algorithm.
///
/// This function follows roughly the same step as the `grisubal` algorithm, but differs in two
/// instances:
/// - The overlapping grid avoids landing Points of Interest on all edges of the grid (instead of
///   only corners), meaning we do not remove any from the original geometry.
/// - Points of Interest are used to associate `VertexAnchor::Node` values to some vertices of the mesh.
///
/// A complete classification of the resulting mesh can be obtained using the [`classify_capture`] function.
///
/// # Return / Errors
///
/// This function returns a `Result` taking the following values:
/// - `Ok(CMap2)` -- Algorithm ran successfully.
/// - `Err(GrisubalError)` -- Algorithm encountered an issue. See [`GrisubalError`] for all
///   possible errors.
///
/// # Panics
///
/// This function may panic if the specified VTK file cannot be opened.
#[allow(clippy::needless_pass_by_value)]
pub fn capture_geometry<T: CoordsFloat>(
    file_path: impl AsRef<std::path::Path>,
    grid_cell_sizes: [T; 2],
    clip: Clip,
) -> Result<CMap2<T>, GrisubalError> {
    // --- IMPORT VTK INPUT
    let geometry_vtk = match Vtk::import(file_path) {
        Ok(vtk) => vtk,
        Err(e) => panic!("E: could not open specified vtk file - {e}"),
    };

    // --- BUILD OUR MODEL FROM THE VTK IMPORT
    let geometry = Geometry2::try_from(geometry_vtk)?;

    // --- FIRST DETECTION OF ORIENTATION ISSUES
    detect_orientation_issue(&geometry)?;

    // --- FIND AN OVERLAPPING GRID
    let ([nx, ny], origin) = compute_overlapping_grid(&geometry, grid_cell_sizes, true)?;
    let [cx, cy] = grid_cell_sizes;

    // --- BUILD THE GRID
    let mut cmap = GridBuilder::<2, T>::default()
        .n_cells([nx, ny])
        .len_per_cell([cx, cy])
        .origin([origin.0, origin.1])
        .add_attribute::<Boundary>() // will be used for clipping
        .add_attribute::<VertexAnchor>()
        .add_attribute::<EdgeAnchor>()
        .add_attribute::<FaceAnchor>()
        .build()
        .expect("E: unreachable"); // unreachable because grid dims are valid

    // process the geometry

    // --- STEP 1 & 2
    // (1)
    let (new_segments, intersection_metadata) =
        generate_intersection_data(&cmap, &geometry, [nx, ny], [cx, cy], origin);
    // (2)
    let n_intersec = intersection_metadata.len();
    let (edge_intersec, dart_slices) =
        group_intersections_per_edge(&mut cmap, intersection_metadata);
    let intersection_darts = compute_intersection_ids(n_intersec, &edge_intersec, &dart_slices);

    // --- STEP 3
    insert_intersections(&cmap, &edge_intersec, &dart_slices);

    // --- STEP 4
    let edges = generate_edge_data(&cmap, &geometry, &new_segments, &intersection_darts);

    // --- STEP 5
    insert_edges_in_map(&mut cmap, &edges);

    // --- CLIP
    match clip {
        Clip::Left => clip_left(&mut cmap)?,
        Clip::Right => clip_right(&mut cmap)?,
        Clip::None => {}
    }

    // CLEANUP
    cmap.remove_attribute_storage::<Boundary>();

    Ok(cmap)
}

/// Error-modeling enum for classification issues.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ClassificationError {
    /// One of the attribute used to classify mesh entities isn't included in the map.
    #[error("missing attribute for classification: {0}")]
    MissingAttribute(&'static str),
    /// There is an unsupported configuration in the geometry.
    #[error("geometry contains an ambiguous or unsupported configuration: {0}")]
    UnsupportedGeometry(&'static str),
}

/// Classify all entities of a mesh on a default geometry.
///
/// This function classifies all i-cells of a mesh using a basic traversal algorithm. It expects
/// the map passed as argument to already have a set of vertices linked to nodes.
///
/// The algorithm uses a first oriented traversal starting from anchored vertices to classifies
/// boundaries as curves. It also checks for curves that are not linked to anchored vertices (e.g.
/// a hole inside of the geometry).
///
/// Once curves are classified, the remaining unclassified cells are associated to surfaces, which
/// are identified using a BFS-like algorithm stopping on already marked edges, to correctly
/// discriminate surfaces.
///
/// # Errors
///
/// This function may fail and return an error if:
/// - one of the attribute used to classify entities is missing (e.g. `VertexAnchor`),
/// - The structure of the mesh is incorrect / unsupported (e.g. an open geometry).
///
/// # Panics
///
/// In `debug` mode, we use assertions to check every single i-cell of the mesh has been
/// classified. If that is not the case, the function fill panic.
#[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
pub fn classify_capture<T: CoordsFloat>(cmap: &CMap2<T>) -> Result<(), ClassificationError> {
    if !cmap.contains_attribute::<VertexAnchor>() {
        Err(ClassificationError::MissingAttribute(
            std::any::type_name::<VertexAnchor>(),
        ))?;
    }
    if !cmap.contains_attribute::<EdgeAnchor>() {
        Err(ClassificationError::MissingAttribute(
            std::any::type_name::<EdgeAnchor>(),
        ))?;
    }
    if !cmap.contains_attribute::<FaceAnchor>() {
        Err(ClassificationError::MissingAttribute(
            std::any::type_name::<FaceAnchor>(),
        ))?;
    }

    let mut curve_id = 0;

    // classify boundaries
    for (i, dart) in cmap
        .iter_vertices()
        .filter_map(|v| {
            cmap.force_read_attribute::<VertexAnchor>(v).and_then(|_| {
                cmap.orbit(OrbitPolicy::Vertex, v)
                    .find(|d| cmap.beta::<2>(*d) == NULL_DART_ID)
            })
        })
        .enumerate()
    {
        mark_curve(cmap, dart, i as CurveIdType)?;
        curve_id = curve_id.max(i);
    }
    // check for boundaries that are not reachable from anchored vertices (e.g. a hole with no PoI)
    while let Some(dart) = (1..cmap.n_darts() as DartIdType)
        .filter_map(|d| {
            // check only used darts on the boundary
            let used = !cmap.is_unused(d);
            if used {
                cmap.orbit(OrbitPolicy::Vertex, d)
                    .find(|dd| cmap.beta::<2>(*dd) == NULL_DART_ID)
            } else {
                None
            }
        })
        .find(|d| {
            cmap.force_read_attribute::<EdgeAnchor>(cmap.edge_id(*d))
                .is_none()
        })
    {
        curve_id += 1; // use a new curve id
        cmap.force_write_attribute(
            cmap.vertex_id(dart),
            VertexAnchor::Curve(curve_id as CurveIdType),
        );
        mark_curve(cmap, dart, curve_id as CurveIdType)?;
    }

    // classify inner entities using a coloring-like algorithm
    let mut surface_id = 0;
    let mut queue = VecDeque::new();
    let mut marked = HashSet::new();
    marked.insert(0);
    cmap.iter_faces()
        // does this filter item updated in the for_each block?
        // .filter(|f| cmap.force_read_attribute::<FaceAnchor>(*f).is_some())
        .for_each(|f| {
            if cmap.force_read_attribute::<FaceAnchor>(f).is_none() {
                queue.clear();
                queue.push_front(f);
                while let Some(crt) = queue.pop_front() {
                    // if the filter works correctly, this if isn't useful
                    cmap.force_write_attribute(crt, FaceAnchor::Surface(surface_id));
                    cmap.orbit(OrbitPolicy::Face, crt as DartIdType)
                        .filter(|d| {
                            cmap.force_read_attribute::<EdgeAnchor>(cmap.edge_id(*d))
                                .is_none()
                        })
                        .for_each(|d| {
                            cmap.force_write_attribute(
                                cmap.edge_id(d),
                                EdgeAnchor::Surface(surface_id),
                            );
                            if cmap
                                .force_read_attribute::<VertexAnchor>(cmap.vertex_id(d))
                                .is_none()
                            {
                                cmap.force_write_attribute(
                                    cmap.vertex_id(d),
                                    VertexAnchor::Surface(surface_id),
                                );
                            }
                            let neighbor_face = cmap.face_id(cmap.beta::<2>(d));
                            if marked.insert(neighbor_face) {
                                queue.push_back(neighbor_face);
                            }
                        });
                }
                surface_id += 1;
            }
        });

    // in debug mode, ensure all entities are classified
    debug_assert!(
        cmap.iter_vertices()
            .all(|v| cmap.force_read_attribute::<VertexAnchor>(v).is_some()),
        "E: Not all vertices are classified",
    );
    debug_assert!(
        cmap.iter_edges()
            .all(|e| cmap.force_read_attribute::<EdgeAnchor>(e).is_some()),
        "E: Not all edges are classified",
    );
    debug_assert!(
        cmap.iter_faces()
            .all(|f| cmap.force_read_attribute::<FaceAnchor>(f).is_some()),
        "E: Not all faces are classified",
    );

    Ok(())
}

/// Traverse and anchor a boundary starting from the specified dart.
///
/// All entities making up the boundary (vertices, edges) are anchored to the `curve_id` curve.
/// The only exception is the vertex associated to the starting dart.
#[allow(clippy::cast_possible_truncation)]
fn mark_curve<T: CoordsFloat>(
    cmap: &CMap2<T>,
    start: DartIdType,
    curve_id: CurveIdType,
) -> Result<(), ClassificationError> {
    // only write attribute on the edge for the first one
    // since we start from an anchored vertex
    cmap.force_write_attribute::<EdgeAnchor>(cmap.edge_id(start), EdgeAnchor::Curve(curve_id));
    let mut next = cmap.beta::<1>(start);
    while cmap
        .force_read_attribute::<VertexAnchor>(cmap.vertex_id(next))
        .is_none()
    {
        if let Some(crt) = cmap
            .orbit(OrbitPolicy::Vertex, next)
            .find(|d| cmap.beta::<2>(*d) == NULL_DART_ID)
        {
            cmap.force_write_attribute(cmap.vertex_id(crt), VertexAnchor::Curve(curve_id));
            cmap.force_write_attribute(cmap.edge_id(crt), EdgeAnchor::Curve(curve_id));
            next = cmap.beta::<1>(crt);
        } else {
            // this should be unreachable as long as the geometry is closed and the node is on the boundary
            Err(ClassificationError::UnsupportedGeometry(
                "open geometry or node outside of boundary",
            ))?;
        }
    }
    Ok(())
}
