use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_egui::egui::Window;
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_mod_picking::picking_core::PickingPluginsSettings;
use bevy_mod_picking::selection::SelectionPluginSettings;
use egui_dock::egui;

use crate::components::{Beta, Dart, DartId, Edge, EdgeId, Face, FaceId, Vertex, VertexId, Volume};
use crate::resources::{
    BetaRenderColor, BetaWidth, DartHeadMul, DartRenderColor, DartShrink, DartWidth,
    EdgeRenderColor, EdgeWidth, FaceRenderColor, FaceShrink, VertexRenderColor, VertexWidth,
    VolumeRenderColor, VolumeShrink,
};

// --- plugin

/// Plugin used to build the interface.
pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .insert_resource(UiState::default())
            .insert_resource(WindowVisible(false))
            .add_systems(
                Update,
                toggle_window_system.run_if(input_just_pressed(KeyCode::Tab)),
            )
            .add_systems(Update, ui_system);
    }
}

// --- system

#[derive(Resource)]
pub struct WindowVisible(pub bool);

fn toggle_window_system(mut window_visible: ResMut<WindowVisible>) {
    window_visible.0 = !window_visible.0;
}

#[allow(clippy::too_many_arguments)]
fn ui_system(
    mut contexts: EguiContexts,
    window_visible: Res<WindowVisible>,
    drc: ResMut<DartRenderColor>,
    ds: ResMut<DartShrink>,
    dw: ResMut<DartWidth>,
    dhm: ResMut<DartHeadMul>,
    brc: ResMut<BetaRenderColor>,
    bw: ResMut<BetaWidth>,
    verc: ResMut<VertexRenderColor>,
    vew: ResMut<VertexWidth>,
    edrc: ResMut<EdgeRenderColor>,
    edw: ResMut<EdgeWidth>,
    farc: ResMut<FaceRenderColor>,
    fas: ResMut<FaceShrink>,
    vorc: ResMut<VolumeRenderColor>,
    vos: ResMut<VolumeShrink>,
) {
    if window_visible.0 {
        Window::new("Rendering Options")
            .collapsible(false)
            .show(contexts.ctx_mut(), |ui| {
                draw_options(
                    ui, drc, ds, dw, dhm, brc, bw, verc, vew, edrc, edw, farc, fas, vorc, vos,
                );
            });
    }
}

pub fn is_window_open(window_visible: Res<WindowVisible>) -> bool {
    window_visible.0
}

// --- structs

#[derive(Resource)]
pub struct UiState {
    pub(crate) selected_entities: HashSet<Entity>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected_entities: HashSet::new(),
        }
    }
}

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

// --- picking

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

/// Inspection panel drawing function.
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
        if world.get::<Dart>(*entity).is_some() {
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
            egui::Grid::new(format!("dart #{}", id.0)) // need a unique id
                .num_columns(4)
                .show(ui, |ui| {
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
