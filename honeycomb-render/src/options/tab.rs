use crate::{
    BetaRenderColor, BetaWidth, DartHeadMul, DartRenderColor, DartShrink, DartWidth,
    EdgeRenderColor, EdgeWidth, FaceRenderColor, FaceShrink, VertexRenderColor, VertexWidth,
    VolumeRenderColor, VolumeShrink,
};
use bevy::prelude::*;
use bevy_mod_picking::picking_core::PickingPluginsSettings;
use bevy_mod_picking::selection::SelectionPluginSettings;
use egui_dock::egui;

// --- main function

pub fn draw_options(ui: &mut egui::Ui, world: &mut World) {
    draw_map_options(ui, world);

    ui.separator(); // ---

    draw_picking_options(ui, world);
}

// --- map options

macro_rules! opt_rendercol {
    ($ui: ident, $world: ident, $param: ident) => {{
        let mut rendercol = $world.resource_mut::<$param>();
        $ui.checkbox(&mut rendercol.0, "");
        $ui.add_enabled_ui(rendercol.0, |ui| draw_color_picker(ui, &mut rendercol.1));
    }};
}

macro_rules! opt_dragvalue {
    ($ui: ident, $world: ident, $param: ident) => {{
        let mut val = $world.resource_mut::<$param>();
        $ui.add(egui::DragValue::new(&mut val.0).speed(0.01));
    }};
}

fn draw_map_options(ui: &mut egui::Ui, world: &mut World) {
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
            opt_rendercol!(ui, world, DartRenderColor);
            opt_dragvalue!(ui, world, DartShrink);
            opt_dragvalue!(ui, world, DartWidth);
            opt_dragvalue!(ui, world, DartHeadMul);
            ui.end_row();
            // betas
            ui.label("Beta Functions");
            opt_rendercol!(ui, world, BetaRenderColor);
            ui.label("");
            opt_dragvalue!(ui, world, BetaWidth);
            ui.end_row();
            // vertices
            ui.label("Vertices");
            opt_rendercol!(ui, world, VertexRenderColor);
            ui.label("");
            opt_dragvalue!(ui, world, VertexWidth);
            ui.end_row();
            // edges
            ui.label("Edges");
            opt_rendercol!(ui, world, EdgeRenderColor);
            ui.label("");
            opt_dragvalue!(ui, world, EdgeWidth);
            ui.end_row();
            // faces
            ui.label("Faces");
            opt_rendercol!(ui, world, FaceRenderColor);
            opt_dragvalue!(ui, world, FaceShrink);
            ui.end_row();
            // faces
            ui.label("Volumes");
            opt_rendercol!(ui, world, VolumeRenderColor);
            opt_dragvalue!(ui, world, VolumeShrink);
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
