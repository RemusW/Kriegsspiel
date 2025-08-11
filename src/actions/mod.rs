use bevy::prelude::*;

pub mod action;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(action::plugin);
}
