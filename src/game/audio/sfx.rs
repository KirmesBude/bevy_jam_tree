use bevy::{audio::PlaybackMode, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.observe(play_sfx);
}

fn play_sfx(trigger: Trigger<PlaySfx>, mut commands: Commands) {
    let sfx = match trigger.event() {
        PlaySfx::Handle(handle) => handle.clone_weak(),
    };
    commands.spawn(AudioSourceBundle {
        source: sfx,
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    });
}

/// Trigger this event to play a single sound effect.
#[derive(Event)]
pub enum PlaySfx {
    Handle(Handle<AudioSource>),
}
