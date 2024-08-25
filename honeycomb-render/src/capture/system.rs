use crate::{
    CaptureList, DartHeadHandle, DartHeadMul, DartMatHandle, DartRenderColor, DartShrink,
    DartWidth, EdgeMatHandle, EdgeRenderColor, EdgeWidth, FaceMatHandle, FaceNormals,
    FaceRenderColor, FaceShrink, FocusedCapture, MapVertices, VertexHandle, VertexMatHandle,
    VertexRenderColor, VertexWidth,
};
use bevy::color::Color;
use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_mod_picking::PickableBundle;

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn populate_darts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    focused_capture: Res<FocusedCapture>,
    captures: Res<CaptureList>,
    dart_render_color: Res<DartRenderColor>,
    dart_head_mul: Res<DartHeadMul>,
    dart_width: Res<DartWidth>,
    dart_shrink: Res<DartShrink>,
) {
    let head_shape = Cone {
        radius: dart_head_mul.0 * dart_width.0 / 2.,
        height: dart_head_mul.0 * dart_width.0 / 2.,
    };
    let dart_head_handle = meshes.add(head_shape);
    let dart_mat = materials.add(Color::Srgba(Srgba::from_u8_array(
        dart_render_color.1.to_array(),
    )));
    for capture in &captures.0 {
        let vertices = &capture.vertex_vals;
        let normals = &capture.normals;
        let visibility =
            if focused_capture.0 .0 == capture.metadata.capture_id && dart_render_color.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        for (head, body) in &capture.darts {
            let face_id = head.face_id.0;
            let (n1, n2) = (
                &normals[&face_id][body.dart_body.normals.0],
                &normals[&face_id][body.dart_body.normals.1],
            );
            let (ov1, ov2) = (
                &vertices[body.dart_body.vertices.0],
                &vertices[body.dart_body.vertices.1],
            );
            let (v1, v2) = (*ov1 + (*n1 * dart_shrink.0), *ov2 + (*n2 * dart_shrink.0));
            let (dir, len) = ((v2 - v1).normalize(), (v2 - v1).length());

            let mut transform = Transform::from_translation((v1 + v2) / 2.);
            transform.rotation = if dir == Vec3::Y {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_arc(dir, Vec3::Y)
            };
            // dart body
            commands
                .spawn((
                    body.clone(),
                    PbrBundle {
                        mesh: meshes.add(Cylinder::new(
                            dart_width.0,
                            // FIXME: clunky
                            len * (1. - dart_shrink.0.abs()),
                        )),
                        material: dart_mat.clone(),
                        transform,
                        visibility,
                        ..Default::default()
                    },
                    PickableBundle::default(),
                ))
                .insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: false,
                        colour: Color::WHITE,
                        width: 1.0,
                    },
                    ..default()
                });
            // dart head
            // FIXME: clunky
            let mut transform_head = Transform::from_translation(
                (v1 + v2 + dir * len * (1. - dart_shrink.0.abs())) / 2.,
            );
            transform_head.rotation = Quat::from_rotation_arc(Vec3::Y, dir);
            commands
                .spawn((
                    head.clone(),
                    PbrBundle {
                        mesh: dart_head_handle.clone(),
                        material: dart_mat.clone(),
                        transform: transform_head,
                        visibility,
                        ..Default::default()
                    },
                    PickableBundle::default(),
                ))
                .insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: false,
                        colour: Color::WHITE,
                        width: 1.0,
                    },
                    ..default()
                });
        }
        commands.insert_resource(MapVertices(vertices.clone()));
        commands.insert_resource(FaceNormals(normals.clone()));
    }
    commands.insert_resource(DartHeadHandle(dart_head_handle));
    commands.insert_resource(DartMatHandle(dart_mat));
}

#[allow(clippy::needless_pass_by_value, unused)]
pub fn populate_beta() {
    todo!()
}

#[allow(clippy::needless_pass_by_value)]
pub fn populate_vertices(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    focused_capture: Res<FocusedCapture>,
    captures: Res<CaptureList>,
    vertex_render_color: Res<VertexRenderColor>,
    vertex_width: Res<VertexWidth>,
) {
    let vertex_handle = meshes.add(Sphere::new(vertex_width.0 / 2.));
    let vertex_mat = materials.add(Color::Srgba(Srgba::from_u8_array(
        vertex_render_color.1.to_array(),
    )));
    for capture in &captures.0 {
        let vertices = &capture.vertex_vals;
        let visibility =
            if focused_capture.0 .0 == capture.metadata.capture_id && vertex_render_color.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        // insert vertices
        for vertex in &capture.vertices {
            commands
                .spawn((
                    vertex.clone(),
                    PbrBundle {
                        mesh: vertex_handle.clone(),
                        material: vertex_mat.clone(),
                        transform: Transform::from_translation(vertices[vertex.vertex.0]),
                        visibility,
                        ..Default::default()
                    },
                    PickableBundle::default(),
                ))
                .insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: false,
                        colour: Color::WHITE,
                        width: 1.0,
                    },
                    ..default()
                });
        }
    }
    commands.insert_resource(VertexHandle(vertex_handle));
    commands.insert_resource(VertexMatHandle(vertex_mat));
}

#[allow(clippy::needless_pass_by_value)]
pub fn populate_edges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    focused_capture: Res<FocusedCapture>,
    captures: Res<CaptureList>,
    edge_render_color: Res<EdgeRenderColor>,
    edge_width: Res<EdgeWidth>,
) {
    let edge_mat = materials.add(Color::Srgba(Srgba::from_u8_array(
        edge_render_color.1.to_array(),
    )));
    for capture in &captures.0 {
        let vertices = &capture.vertex_vals;
        let visibility =
            if focused_capture.0 .0 == capture.metadata.capture_id && edge_render_color.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        for edge in &capture.edges {
            let v1 = &vertices[edge.edge.0]; // == translation
            let v2 = &vertices[edge.edge.1];
            let (dir, len) = ((*v2 - *v1).normalize(), (*v2 - *v1).length());
            let mut transform = Transform::from_translation((*v1 + *v2) / 2.);
            transform.rotation = if dir == Vec3::Y {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_arc(dir, Vec3::Y)
            };
            commands
                .spawn((
                    edge.clone(),
                    PbrBundle {
                        mesh: meshes.add(Cylinder::new(edge_width.0 / 2., len)),
                        material: edge_mat.clone(),
                        transform,
                        visibility,
                        ..Default::default()
                    },
                    PickableBundle::default(),
                ))
                .insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: false,
                        colour: Color::WHITE,
                        width: 1.0,
                    },
                    ..default()
                });
        }
    }
    commands.insert_resource(EdgeMatHandle(edge_mat));
}

#[allow(clippy::needless_pass_by_value, unused)]
pub fn populate_faces(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    focused_capture: Res<FocusedCapture>,
    captures: Res<CaptureList>,
    face_render_color: Res<FaceRenderColor>,
    face_shrink: Res<FaceShrink>,
) {
    let face_mat = materials.add(Color::Srgba(Srgba::from_u8_array(
        face_render_color.1.to_array(),
    )));
    for capture in &captures.0 {
        let vertices = &capture.vertex_vals;
        let normals = &capture.normals;
        let visibility =
            if focused_capture.0 .0 == capture.metadata.capture_id && face_render_color.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        for face in &capture.faces {
            let loc_n = &normals[&face.id.0];
            let ovs = face
                .face
                .0
                .iter()
                .map(|id| vertices[*id])
                .collect::<Vec<_>>();
            let nvs: Vec<_> = loc_n
                .iter()
                .zip(ovs.iter())
                .map(|(vertex, normal)| (*vertex + *normal * face_shrink.0).truncate())
                .collect();
        }
    }
    commands.insert_resource(FaceMatHandle(face_mat));
}
