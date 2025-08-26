//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
    ui::UiDebugOptions, window::PrimaryWindow,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, show_cursor_window);

    // Log `Screen` state transitions.
    // app.add_systems(Update, log_transitions::<Screen>);

    // // Toggle the debug overlay for UI.
    // app.add_systems(
    //     Update,
    //     toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    // );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn show_cursor_window(
    mut contexts: EguiContexts,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
) {
    let (camera, camera_transform) = *camera_query;
    let Ok(window) = window.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    if let Ok(context) = contexts.ctx_mut() {
        egui::Window::new("Mouse Position").show(context, |ui| {
            ui.label(format!("X: {:.1}, Y: {:.1}", world_pos.x, world_pos.y));
        });
    }
}
