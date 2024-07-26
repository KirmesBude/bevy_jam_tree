//! Game mechanics and content.

use bevy::prelude::*;

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
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct Score(pub usize);
