//! Game mechanics and content.

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use season::Season;

use crate::screen::Screen;

pub mod assets;
pub mod audio;
pub mod season;
pub mod spawn;
pub mod ui;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(EntropyPlugin::<WyRand>::default());
    app.add_plugins((
        audio::plugin,
        assets::plugin,
        spawn::plugin,
        season::plugin,
        ui::plugin,
    ));

    app.init_resource::<Score>();
    app.register_type::<Score>();

    app.add_systems(Update, game_over.run_if(in_state(Screen::Playing)));
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct Score(pub usize);

fn game_over(season: Res<Season>, mut next_screen: ResMut<NextState<Screen>>) {
    if season.year == 3 {
        next_screen.set(Screen::GameOver);
    }
}
