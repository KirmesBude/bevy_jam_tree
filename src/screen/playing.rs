//! The screen state for the main game loop.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::Screen;
use crate::game::{
    assets::SoundtrackAssets, audio::soundtrack::PlaySoundtrack, season::Season,
    spawn::level::SpawnLevel, ui::SpawnGameUi, Score,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Playing).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn enter_playing(
    mut commands: Commands,
    soundtrack_assets: Res<SoundtrackAssets>,
    mut score: ResMut<Score>,
    mut season: ResMut<Season>,
) {
    commands.trigger(SpawnGameUi);
    commands.trigger(SpawnLevel);
    commands.trigger(PlaySoundtrack::Handle(
        soundtrack_assets.gameplay.clone_weak(),
    ));

    *score = Score::default();
    *season = Season::default();
}

fn exit_playing(mut commands: Commands) {
    // We could use [`StateScoped`] on the sound playing entites instead.
    commands.trigger(PlaySoundtrack::Disable);
}

fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
