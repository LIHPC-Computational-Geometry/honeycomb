use crate::capture::ecs_data::{DartBody, DartId, EdgeId, VertexId};
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

    for entity in selected_entities {
        ui.separator();
        if world.get::<DartBody>(*entity).is_some() {
            let Some(id) = world.get::<DartId>(*entity) else {
                unreachable!()
            };
            ui.label(format!("Dart #{}", id.0));
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
