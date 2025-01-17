//! A credits screen that can be accessed from the title screen.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{assets::SoundtrackAssets, audio::soundtrack::PlaySoundtrack},
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Credits), enter_credits);
    app.add_systems(OnExit(Screen::Credits), exit_credits);

    app.add_systems(
        Update,
        handle_credits_action.run_if(in_state(Screen::Credits)),
    );
    app.register_type::<CreditsAction>();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum CreditsAction {
    Back,
}

fn enter_credits(mut commands: Commands, soundtrack_assets: Res<SoundtrackAssets>) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Credits))
        .with_children(|children| {
            children.header("Made by");
            children.label("KirmesBude");

            children.header("Assets");
            children.label("Bevy logo - All rights reserved by the Bevy Foundation. Permission granted for splash screen use when unmodified.");
            children.label("Music - CC BY 4.0 by Vindsvept");

            children.header("Plugins");
            children.label("bevy_rand - by Bluefinger");
            children.label("bevy-inspector-egui - by jakobhellermann");
            children.label("bevy_ecs_tilemap - by StarArawn");

            children.button("Back").insert(CreditsAction::Back);
        });

    commands.trigger(PlaySoundtrack::Handle(
        soundtrack_assets.credits.clone_weak(),
    ));
}

fn exit_credits(mut commands: Commands) {
    commands.trigger(PlaySoundtrack::Disable);
}

fn handle_credits_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&CreditsAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                CreditsAction::Back => next_screen.set(Screen::Title),
            }
        }
    }
}
