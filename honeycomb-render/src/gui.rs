use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_egui::egui::Window;
use bevy_egui::{EguiContexts, EguiPlugin};

use crate::resources::{
    BetaRenderColor, BetaWidth, DartHeadMul, DartRenderColor, DartShrink, DartWidth,
    EdgeRenderColor, EdgeWidth, FaceRenderColor, FaceShrink, VertexRenderColor, VertexWidth,
    VolumeRenderColor, VolumeShrink,
};
use crate::systems::{draw_inspected_data, draw_options};

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
