//! Game mechanics and content.

use bevy::prelude::*;
use spawn::tree::Tree;

use crate::screen::Screen;

pub mod assets;
pub mod audio;
pub mod season;
pub mod spawn;
pub mod ui;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        audio::plugin,
        assets::plugin,
        spawn::plugin,
        season::plugin,
        ui::plugin,
    ));

    app.init_resource::<Score>();
    app.register_type::<Score>();
    app.register_type::<RunGameLogic>();
    app.observe(run_game_logic);
    app.add_systems(
        Update,
        (increment_score, game_over).run_if(in_state(Screen::Playing)),
    );
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct Score(pub usize);

fn increment_score(mut score: ResMut<Score>, query: Query<(), Added<Tree>>) {
    score.0 += query.iter().count();
}

fn game_over(mut next_screen: ResMut<NextState<Screen>>, trees: Query<(), With<Tree>>) {
    if trees.is_empty() {
        next_screen.set(Screen::GameOver);
    }
}

#[derive(Debug, Default, Reflect, Event)]
pub struct RunGameLogic;

fn run_game_logic(_trigger: Trigger<RunGameLogic>) {
    println!("Game logic run")
}
