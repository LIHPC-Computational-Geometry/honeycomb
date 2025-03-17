use bevy::math::{Quat, Vec3};
use bevy::prelude::*;

use crate::capture::FocusedCapture;
use crate::capture::ecs_data::{
    CaptureId, DartBody, DartHead, DartId, Edge, FaceId, FaceNormals, MapVertices, Vertex,
};
use crate::options::resource::{
    DartHeadHandle, DartHeadMul, DartMatHandle, DartRenderColor, DartShrink, DartWidth,
    EdgeMatHandle, EdgeRenderColor, EdgeWidth, VertexHandle, VertexMatHandle, VertexRenderColor,
    VertexWidth,
};

// --- darts

#[allow(clippy::missing_panics_doc)]
/// Dart material handle update system.
pub fn dart_mat_handle(
    mut materials: ResMut<Assets<StandardMaterial>>,
    handle: Res<DartMatHandle>,
    render_color: Res<DartRenderColor>,
) {
    let mat = materials.get_mut(&handle.0).expect("unreachable");
    mat.base_color = Color::Srgba(Srgba::from_u8_array(render_color.1.to_array()));
}

/// Dart render color and status update system.
pub fn dart_render(
    mut dart_comps: Query<(&CaptureId, &mut Visibility), With<DartId>>, // with dart_id == heads & bodies
    focused_capture: Res<FocusedCapture>,
    render_color: Res<DartRenderColor>,
) {
    dart_comps
        .par_iter_mut()
        .for_each(|(cap_id, mut visibility)| {
            *visibility.as_mut() = if render_color.0 && (focused_capture.0 == *cap_id) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            }
        });
}

#[allow(clippy::missing_panics_doc)]
/// Dart head mesh handle update system.
pub fn dart_heads_handle(
    mut meshes: ResMut<Assets<Mesh>>,
    handle: Res<DartHeadHandle>,
    dart_width: Res<DartWidth>,
    dart_head_mul: Res<DartHeadMul>,
) {
    let mesh = meshes.get_mut(&handle.0).expect("unreachable");
    let new_shape = Cone {
        radius: dart_head_mul.0 * dart_width.0 / 2.,
        height: dart_head_mul.0 * dart_width.0 / 2.,
    };
    *mesh = new_shape.into();
}

/// Dart head transform update system.
pub fn dart_heads_transform(
    mut heads: Query<(&mut Transform, &DartHead, &FaceId)>,
    vertices: Res<MapVertices>,
    normals: Res<FaceNormals>,
    dart_shrink: Res<DartShrink>,
) {
    heads
        .par_iter_mut()
        .for_each(|(mut transform, head, face_id)| {
            let (n1, n2) = (
                &normals.0[&face_id.0][head.normals.0],
                &normals.0[&face_id.0][head.normals.1],
            );
            let (ov1, ov2) = (&vertices.0[head.vertices.0], &vertices.0[head.vertices.1]);
            let (v1, v2) = (*ov1 + (*n1 * dart_shrink.0), *ov2 + (*n2 * dart_shrink.0));
            let (dir, len) = ((v2 - v1).normalize(), (v2 - v1).length());

            transform.translation = (v1 + v2 + dir * len * (1. - dart_shrink.0.abs())) / 2.;
            transform.rotation = if dir == Vec3::Y {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_arc(Vec3::Y, dir)
            };
        });
}

#[allow(clippy::missing_panics_doc)]
/// Dart body mesh handles update system.
pub fn dart_bodies_handles(
    mut meshes: ResMut<Assets<Mesh>>,
    mut bodies: Query<(&Handle<Mesh>, &DartBody, &FaceId)>,
    vertices: Res<MapVertices>,
    normals: Res<FaceNormals>,
    dart_shrink: Res<DartShrink>,
    dart_width: Res<DartWidth>,
) {
    bodies.iter_mut().for_each(|(handle, body, face_id)| {
        let (n1, n2) = (
            &normals.0[&face_id.0][body.normals.0],
            &normals.0[&face_id.0][body.normals.1],
        );
        let (ov1, ov2) = (&vertices.0[body.vertices.0], &vertices.0[body.vertices.1]);
        let (v1, v2) = (*ov1 + (*n1 * dart_shrink.0), *ov2 + (*n2 * dart_shrink.0));
        let len = (v2 - v1).length();

        let mesh = meshes.get_mut(handle).expect("unreachable");
        *mesh = Cylinder::new(
            dart_width.0 / 2.,
            // FIXME: clunky
            len * (1. - dart_shrink.0.abs()),
        )
        .into();
    });
}

/// Dart body transform update system.
pub fn dart_bodies_transform(
    mut bodies: Query<(&mut Transform, &DartBody, &FaceId)>,
    vertices: Res<MapVertices>,
    normals: Res<FaceNormals>,
    dart_shrink: Res<DartShrink>,
) {
    bodies
        .par_iter_mut()
        .for_each(|(mut transform, body, face_id)| {
            let (n1, n2) = (
                &normals.0[&face_id.0][body.normals.0],
                &normals.0[&face_id.0][body.normals.1],
            );
            let (ov1, ov2) = (&vertices.0[body.vertices.0], &vertices.0[body.vertices.1]);
            let (v1, v2) = (*ov1 + (*n1 * dart_shrink.0), *ov2 + (*n2 * dart_shrink.0));
            let dir = (v2 - v1).normalize();

            transform.translation = (v1 + v2) / 2.;
            transform.rotation = if dir == Vec3::Y {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_arc(Vec3::Y, dir)
            };
        });
}

// --- vertices

/// Vertex render color and status update system.
pub fn vertices_render(
    mut query: Query<(&CaptureId, &mut Visibility), With<Vertex>>,
    focused_capture: Res<FocusedCapture>,
    render_color: Res<VertexRenderColor>,
) {
    query.par_iter_mut().for_each(|(cap_id, mut visibility)| {
        *visibility.as_mut() = if render_color.0 && (focused_capture.0 == *cap_id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        }
    });
}

#[allow(clippy::missing_panics_doc)]
/// Vertex mesh handle update system.
pub fn vertices_handle(
    mut meshes: ResMut<Assets<Mesh>>,
    handle: Res<VertexHandle>,
    vertex_width: Res<VertexWidth>,
) {
    let mesh = meshes.get_mut(&handle.0).expect("unreachable");
    *mesh = Sphere::new(vertex_width.0 / 2.).into();
}

#[allow(clippy::missing_panics_doc)]
/// Vertex material handle update system.
pub fn vertices_mat_handle(
    mut materials: ResMut<Assets<StandardMaterial>>,
    handle: Res<VertexMatHandle>,
    render_color: Res<VertexRenderColor>,
) {
    let mat = materials.get_mut(&handle.0).expect("unreachable");
    mat.base_color = Color::Srgba(Srgba::from_u8_array(render_color.1.to_array()));
}

// --- edges

/// Edge render color and status update system.
pub fn edges_render(
    mut query: Query<(&CaptureId, &mut Visibility), With<Edge>>,
    focused_capture: Res<FocusedCapture>,
    render_color: Res<EdgeRenderColor>,
) {
    query.par_iter_mut().for_each(|(cap_id, mut visibility)| {
        *visibility.as_mut() = if render_color.0 && (focused_capture.0 == *cap_id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        }
    });
}

#[allow(clippy::missing_panics_doc)]
/// Edge mesh handles update system.
pub fn edges_handle(
    mut meshes: ResMut<Assets<Mesh>>,
    handles: Query<(&Handle<Mesh>, &Edge)>,
    vertices: Res<MapVertices>,
    edge_width: Res<EdgeWidth>,
) {
    handles.iter().for_each(|(handle, edge)| {
        let v1 = &vertices.0[edge.0];
        let v2 = &vertices.0[edge.1];
        let len = (*v2 - *v1).length();
        let mesh = meshes.get_mut(handle).expect("unreachable");
        *mesh = Cylinder::new(edge_width.0 / 2., len).into();
    });
}

#[allow(clippy::missing_panics_doc)]
/// Edge material handle update system.
pub fn edges_mat_handle(
    mut materials: ResMut<Assets<StandardMaterial>>,
    handle: Res<EdgeMatHandle>,
    render_color: Res<EdgeRenderColor>,
) {
    let mat = materials.get_mut(&handle.0).expect("unreachable");
    mat.base_color = Color::Srgba(Srgba::from_u8_array(render_color.1.to_array()));
}
