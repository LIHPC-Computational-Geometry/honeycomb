use bevy::prelude::*;

use crate::{
    components::{Dart, Edge, FaceId, Vertex},
    resources::{
        DartHeadMul, DartRenderColor, DartShrink, EdgeRenderColor, FaceNormals, FaceRenderColor,
        FaceShrink, MapVertices, VertexRenderColor, VertexWidth,
    },
};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct DartGizmos;

pub fn render_dart_enabled(drc: Res<DartRenderColor>) -> bool {
    drc.0
}

pub fn render_darts(
    mut gizmos: Gizmos<DartGizmos>,
    // common data
    vertices: Res<MapVertices>,
    face_normals: Res<FaceNormals>,
    // dart render params
    dart_render_color: Res<DartRenderColor>,
    dart_head_mul: Res<DartHeadMul>,
    // dart_width: Res<DartWidth>, // config is edited directly in the option functions
    dart_shrink: Res<DartShrink>,
    // dart to render
    q: Query<(&Dart, &FaceId)>,
) {
    // let dart_mat = materials.add(Color::Srgba(Srgba::from_u8_array(
    //     dart_render_color.1.to_array(),
    // )));
    let vertices = &vertices.0;
    let face_normals = &face_normals.0;
    let [red, green, blue, alpha] = dart_render_color.1.to_srgba_unmultiplied();
    for (d, face_id) in &q {
        let (n1, n2) = (
            &face_normals[&(face_id.0, d.start)],
            &face_normals[&(face_id.0, d.end)],
        );
        let (ov1, ov2) = (&vertices[d.start], &vertices[d.end]);

        let (mut v1, mut v2) = (*ov1 + (*n1 * dart_shrink.0), *ov2 + (*n2 * dart_shrink.0));
        let (dir, len) = ((v2 - v1).normalize(), (v2 - v1).length());
        v1 += dir * (len * dart_shrink.0.abs() / 2.0);
        v2 -= dir * (len * dart_shrink.0.abs() / 2.0);

        gizmos
            .arrow(v1, v2, Color::srgba_u8(red, green, blue, alpha))
            .with_tip_length(dart_head_mul.0);
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct VertexGizmos;

pub fn render_vertex_enabled(vrc: Res<VertexRenderColor>) -> bool {
    vrc.0
}

/// System used to generate vertex entities in the ECS.
pub fn render_vertices(
    mut gizmos: Gizmos<VertexGizmos>,
    vertices: Res<MapVertices>,
    vertex_render_color: Res<VertexRenderColor>,
    vertex_width: Res<VertexWidth>,
    q: Query<&Vertex>,
) {
    let vertices = &vertices.0;
    let [red, green, blue, alpha] = vertex_render_color.1.to_srgba_unmultiplied();

    for v in &q {
        gizmos.sphere(
            vertices[v.0],
            Quat::default(),
            vertex_width.0 / 2.0,
            Color::srgba_u8(red, green, blue, alpha),
        );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct EdgeGizmos;

pub fn render_edge_enabled(erc: Res<EdgeRenderColor>) -> bool {
    erc.0
}

/// System used to generate edge entities in the ECS.
pub fn render_edges(
    mut gizmos: Gizmos<EdgeGizmos>,
    vertices: Res<MapVertices>,
    edge_render_color: Res<EdgeRenderColor>,
    // edge_width: Res<EdgeWidth>,
    q: Query<&Edge>,
) {
    let vertices = &vertices.0;
    let [red, green, blue, alpha] = edge_render_color.1.to_srgba_unmultiplied();

    for e in &q {
        gizmos.line(
            vertices[e.0],
            vertices[e.1],
            Color::srgba_u8(red, green, blue, alpha),
        );
    }
}

/// System used to generate face entities in the ECS.
///
/// NOT YET IMPLEMENTED
#[allow(unused)]
pub fn populate_faces(
    mut gizmos: Gizmos<VertexGizmos>,
    vertices: Res<MapVertices>,
    face_normals: Res<FaceNormals>,
    face_render_color: Res<FaceRenderColor>,
    face_shrink: Res<FaceShrink>,
) {
}
