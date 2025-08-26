use bevy::prelude::*;

use crate::{ menus::Menu, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Main),
        children![
            widget::header("Play"),
            widget::header("Settings"),
            widget::header("Credits"),
            #[cfg(not(target_family = "wasm"))]
            widget::header("Exit"),
        ],
    ));
}
