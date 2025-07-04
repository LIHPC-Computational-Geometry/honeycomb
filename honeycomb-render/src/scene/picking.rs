use bevy::prelude::*;
// use bevy_mod_outline::OutlineVolume;
use bevy_mod_picking::prelude::PickSelection;

use crate::gui::UiState;

/// Picking update routine.
pub fn update_picking(
    mut targets: Query<(Entity, &PickSelection /* , &mut OutlineVolume */)>,
    mut ui_state: ResMut<UiState>,
) {
    let selection = &mut ui_state.selected_entities;

    for (entity, pick_interaction /* , mut outline */) in &mut targets {
        if pick_interaction.is_selected {
            // outline.visible = true;
            let _ = selection.insert(entity);
        } else {
            // outline.visible = false;
            let _ = selection.remove(&entity);
        }
    }
}
