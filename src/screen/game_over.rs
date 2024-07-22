//! Gameover

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{assets::SoundtrackAssets, audio::soundtrack::PlaySoundtrack, Score},
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GameOver), enter_game_over);
    app.add_systems(OnExit(Screen::GameOver), exit_game_over);

    app.add_systems(
        Update,
        handle_gameover_action.run_if(in_state(Screen::GameOver)),
    );
    app.register_type::<GameOverAction>();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum GameOverAction {
    Back,
}

fn enter_game_over(
    score: Res<Score>,
    mut commands: Commands,
    soundtrack_assets: Res<SoundtrackAssets>,
) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::GameOver))
        .with_children(|children| {
            children.header("GAME OVER");

            children.header("Score:");
            children.label(format!("{}", score.0));

            children.button("Back").insert(GameOverAction::Back);
        });

    commands.trigger(PlaySoundtrack::Handle(
        soundtrack_assets.credits.clone_weak(),
    ));
}

fn exit_game_over(mut commands: Commands) {
    commands.trigger(PlaySoundtrack::Disable);
}

fn handle_gameover_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&GameOverAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                GameOverAction::Back => next_screen.set(Screen::Title),
            }
        }
    }
}
