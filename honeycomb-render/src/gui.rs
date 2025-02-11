use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;
use bevy_egui::egui::{Ui, WidgetText};
use bevy_egui::{EguiContext, EguiPlugin, EguiSet};
use egui_dock::{DockArea, DockState, NodeIndex, Style, Tree};

use crate::systems::{draw_inspected_data, draw_options};

// --- plugin

/// Plugin used to build the interface.
pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .insert_resource(UiState::default())
            .add_systems(
                PostUpdate,
                show_ui
                    .before(EguiSet::ProcessOutput)
                    .before(TransformSystem::TransformPropagate),
            );
    }
}

// --- system

fn show_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };

    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut());
    });
}

// --- structs

#[derive(Debug, PartialEq, Eq)]
pub enum CustomTab {
    Render,
    Inspector,
    Options,
}

#[derive(Resource)]
pub struct UiState {
    state: DockState<CustomTab>,
    viewport_rect: bevy_egui::egui::Rect,
    pub(crate) selected_entities: HashSet<Entity>,
}

impl Default for UiState {
    fn default() -> Self {
        let mut state = DockState::new(vec![CustomTab::Render]);
        let tree = state.main_surface_mut();
        let [_render, options_and_inspector] =
            tree.split_left(NodeIndex::root(), 0.3, vec![CustomTab::Options]);
        let [_options, _inspector] =
            tree.split_below(options_and_inspector, 0.5, vec![CustomTab::Inspector]);

        Self {
            state,
            viewport_rect: egui_dock::egui::Rect::NOTHING,
            selected_entities: HashSet::new(),
        }
    }
}

impl UiState {
    pub fn ui(&mut self, world: &mut World, ctx: &mut bevy_egui::egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
        };
        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }

    pub fn tree(&self) -> &Tree<CustomTab> {
        self.state.main_surface()
    }

    pub fn tree_mut(&mut self) -> &mut Tree<CustomTab> {
        self.state.main_surface_mut()
    }
}

pub struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut HashSet<Entity>,
    viewport_rect: &'a mut bevy_egui::egui::Rect,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = CustomTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        format!("{tab:?}").into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            CustomTab::Render => *self.viewport_rect = ui.clip_rect(),
            CustomTab::Inspector => draw_inspected_data(ui, self.world, self.selected_entities),
            CustomTab::Options => draw_options(ui, self.world),
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, CustomTab::Render)
    }
}
