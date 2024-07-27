//! A credits screen that can be accessed from the title screen.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{assets::SoundtrackAssets, audio::soundtrack::PlaySoundtrack},
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::HowToPlay), enter_how_to_play);
    app.add_systems(OnExit(Screen::HowToPlay), exit_how_to_play);

    app.add_systems(
        Update,
        handle_how_to_play_action.run_if(in_state(Screen::HowToPlay)),
    );
    app.register_type::<HowToPlayAction>();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum HowToPlayAction {
    Back,
}

fn enter_how_to_play(mut commands: Commands, soundtrack_assets: Res<SoundtrackAssets>) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::HowToPlay))
        .with_children(|children| {
            children.header("Gain as many points as possible in 3 years:");
            children.label("Each winter you gain 5 points for each mature tree felled and 6 points for each overmature tree felled,");
            children.label("Points are tripled if the corresponding tree is on nutrient soil.");

            children.header("Tree logic:");
            children.label("Apart from winter, trees will always try to grow. They can do so if the level of their 8 neighbour trees does not exceed a level of 2.");
            children.label("Apart from summer, trees can die due to overcrowding. They do so if the level of their 8 neighbour trees exceeds a level of 4.");
            children.label("Seedling, immature and mature are level 1, while overmature is level 2.");

            children.button("Back").insert(HowToPlayAction::Back);
        });

    commands.trigger(PlaySoundtrack::Handle(
        soundtrack_assets.credits.clone_weak(),
    ));
}

fn exit_how_to_play(mut commands: Commands) {
    commands.trigger(PlaySoundtrack::Disable);
}

fn handle_how_to_play_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&HowToPlayAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                HowToPlayAction::Back => next_screen.set(Screen::Title),
            }
        }
    }
}
