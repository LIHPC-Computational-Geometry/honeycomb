use crate::capture::ecs_data::{DartBody, DartId, EdgeId, FaceId, VertexId};
use crate::{Beta, Edge, Face, Vertex, Volume};
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_egui::egui;

pub fn draw_inspected_data(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entities: &HashSet<Entity>,
) {
    ui.label(egui::RichText::new("Selected Components").size(15.));

    if selected_entities.is_empty() {
        ui.separator();
    }

    for entity in selected_entities {
        ui.separator();
        if world.get::<DartBody>(*entity).is_some() {
            let Some(id) = world.get::<DartId>(*entity) else {
                unreachable!()
            };
            ui.label(format!("Dart #{}", id.0));
            let Some(vid) = world.get::<VertexId>(*entity) else {
                unreachable!();
            };
            let Some(eid) = world.get::<EdgeId>(*entity) else {
                unreachable!();
            };
            let Some(fid) = world.get::<FaceId>(*entity) else {
                unreachable!();
            };
            egui::Grid::new("dart_grid").num_columns(4).show(ui, |ui| {
                ui.label("i-cells");
                ui.label("Vertex");
                ui.label("Edge");
                ui.label("Face");
                ui.end_row();
                ui.label("IDs");
                ui.label(format!("{}", vid.0));
                ui.label(format!("{}", eid.0));
                ui.label(format!("{}", fid.0));
            });
        } else if world.get::<Beta>(*entity).is_some() {
            ui.label("Beta");
        } else if world.get::<Vertex>(*entity).is_some() {
            let Some(id) = world.get::<VertexId>(*entity) else {
                unreachable!()
            };
            ui.label(format!("Vertex #{}", id.0));
        } else if world.get::<Edge>(*entity).is_some() {
            let Some(id) = world.get::<EdgeId>(*entity) else {
                unreachable!()
            };
            ui.label(format!("Edge #{}", id.0));
        } else if world.get::<Face>(*entity).is_some() {
            ui.label("Face");
        } else if world.get::<Volume>(*entity).is_some() {
            ui.label("Volume");
        }
    }
}
