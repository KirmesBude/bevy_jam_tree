//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, log::LogPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    game::{
        season::state::NextSeasonState,
        spawn::tree::{DespawnTree, SpawnTree},
    },
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(WorldInspectorPlugin::new());
    app.add_plugins(LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,bevy_jam_tree=debug".into(),
        level: bevy::log::Level::DEBUG,
        ..default()
    });
    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);

    app.add_systems(
        Update,
        (
            log_events::<NextSeasonState>,
            log_events::<SpawnTree>,
            log_events::<DespawnTree>,
        ),
    );
}

fn log_events<T: Event + std::fmt::Debug>(mut event_reader: EventReader<T>) {
    for event in event_reader.read() {
        debug!("{:?}", event);
    }
}
