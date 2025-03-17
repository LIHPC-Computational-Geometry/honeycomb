use bevy::prelude::*;
use bevy_mod_picking::picking_core::PickingPluginsSettings;
use bevy_mod_picking::selection::SelectionPluginSettings;
use egui_dock::egui;

use crate::resources::{
    BetaRenderColor, BetaWidth, DartHeadMul, DartRenderColor, DartShrink, DartWidth,
    EdgeRenderColor, EdgeWidth, FaceRenderColor, FaceShrink, VertexRenderColor, VertexWidth,
    VolumeRenderColor, VolumeShrink,
};

// --- map options

/// rendering options drawing function.
#[allow(clippy::too_many_arguments)]
pub fn draw_options(
    ui: &mut egui::Ui,
    mut drc: ResMut<DartRenderColor>,
    mut ds: ResMut<DartShrink>,
    mut dw: ResMut<DartWidth>,
    mut dhm: ResMut<DartHeadMul>,
    mut brc: ResMut<BetaRenderColor>,
    mut bw: ResMut<BetaWidth>,
    mut verc: ResMut<VertexRenderColor>,
    mut vew: ResMut<VertexWidth>,
    mut edrc: ResMut<EdgeRenderColor>,
    mut edw: ResMut<EdgeWidth>,
    mut farc: ResMut<FaceRenderColor>,
    mut fas: ResMut<FaceShrink>,
    mut vorc: ResMut<VolumeRenderColor>,
    mut vos: ResMut<VolumeShrink>,
) {
    ui.label(egui::RichText::new("Map Rendering").size(15.));
    ui.separator(); // ---

    egui::Grid::new("map_opt_grid")
        .num_columns(6)
        .show(ui, |ui| {
            // column names
            ui.label(egui::RichText::new("Object"));
            ui.label(egui::RichText::new("Render"));
            ui.label(egui::RichText::new("Color"));
            ui.label(egui::RichText::new("Shrink"));
            ui.label(egui::RichText::new("Width"));
            ui.label(egui::RichText::new("Head Factor"));
            ui.end_row();
            // darts
            ui.label("Darts");
            ui.checkbox(&mut drc.0, "");
            ui.add_enabled_ui(drc.0, |ui| draw_color_picker(ui, &mut drc.1));
            ui.add(egui::DragValue::new(&mut ds.0).speed(0.01));
            ui.add(egui::DragValue::new(&mut dw.0).speed(0.01));
            ui.add(egui::DragValue::new(&mut dhm.0).speed(0.01));
            ui.end_row();

            // betas
            ui.label("Beta Functions");
            ui.label("Darts");
            ui.checkbox(&mut brc.0, "");
            ui.add_enabled_ui(brc.0, |ui| draw_color_picker(ui, &mut brc.1));
            ui.label("");
            ui.add(egui::DragValue::new(&mut bw.0).speed(0.01));
            ui.end_row();

            // vertices
            ui.label("Vertices");
            ui.checkbox(&mut verc.0, "");
            ui.add_enabled_ui(verc.0, |ui| draw_color_picker(ui, &mut verc.1));
            ui.label("");
            ui.add(egui::DragValue::new(&mut vew.0).speed(0.01));
            ui.end_row();

            // edges
            ui.label("Edges");
            ui.checkbox(&mut edrc.0, "");
            ui.add_enabled_ui(edrc.0, |ui| draw_color_picker(ui, &mut edrc.1));
            ui.label("");
            ui.add(egui::DragValue::new(&mut edw.0).speed(0.01));
            ui.end_row();

            // faces
            ui.label("Faces");
            ui.checkbox(&mut farc.0, "");
            ui.add_enabled_ui(farc.0, |ui| draw_color_picker(ui, &mut farc.1));
            ui.label("");
            ui.add(egui::DragValue::new(&mut fas.0).speed(0.01));
            ui.end_row();

            // faces
            ui.label("Volumes");
            ui.checkbox(&mut vorc.0, "");
            ui.add_enabled_ui(vorc.0, |ui| draw_color_picker(ui, &mut vorc.1));
            ui.label("");
            ui.add(egui::DragValue::new(&mut vos.0).speed(0.01));
            ui.end_row();
        });
}

fn draw_color_picker(ui: &mut egui::Ui, color: &mut egui::Color32) {
    // create intermediate variable
    let mut egui_color =
        egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), color.a());
    // insert the color picker
    let color_picker = egui::color_picker::color_edit_button_srgba(
        ui,
        &mut egui_color,
        egui::color_picker::Alpha::Opaque,
    );
    // update the original color variable
    if color_picker.changed() {
        *color = egui::Color32::from_rgba_premultiplied(
            egui_color.r(),
            egui_color.g(),
            egui_color.b(),
            egui_color.a(),
        );
    }
}

// --- picking options

#[allow(unused)]
fn draw_picking_options(ui: &mut egui::Ui, world: &mut World) {
    ui.label(egui::RichText::new("Picking").size(15.));
    ui.separator(); // ---

    egui::Grid::new("pick_opt_grid")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Enable");
            {
                let mut picking_params = world.resource_mut::<PickingPluginsSettings>();
                ui.checkbox(&mut picking_params.is_enabled, "");
            }
            ui.end_row();
            ui.label("Click nothing deselect all");
            {
                let mut picking_params = world.resource_mut::<SelectionPluginSettings>();
                ui.checkbox(&mut picking_params.click_nothing_deselect_all, "");
            }
        });
}
