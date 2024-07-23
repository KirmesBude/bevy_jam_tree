use bevy::prelude::*;

use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<(Season, SeasonKind)>();
    app.init_resource::<Season>();

    app.add_systems(
        Update,
        (activate_season, tick_timer).run_if(in_state(Screen::Playing)),
    );
}

#[derive(Debug, Reflect)]
pub enum SeasonKind {
    Spring, /* User action: Plant seeds; Passive effect: No trees die */
    Summer, /* User action: Place/combat(?) wildfires; Passive effects: Accelerated growth, random new trees */
    Autumn, /* User action: Wind direction?; Passive effect: Trees multiply */
    Winter, /* User action: AoE Heavy Snowfall(No trees are felled there); Passive effect: Trees are felled (mature/overmature) */
}

#[derive(Debug, Reflect, Resource)]
#[reflect(Resource)]
pub struct Season {
    active: bool,
    timer: Timer,
    kind: SeasonKind, /* Always growing */
    user_action_resource: usize,
}

impl Default for Season {
    fn default() -> Self {
        Self {
            active: false,
            timer: Timer::from_seconds(20.0, TimerMode::Once),
            kind: SeasonKind::Spring,
            user_action_resource: 1,
        }
    }
}

impl Season {
    pub fn spring() -> Self {
        Self {
            kind: SeasonKind::Spring,
            ..default()
        }
    }

    pub fn summer() -> Self {
        Self {
            kind: SeasonKind::Summer,
            ..default()
        }
    }

    pub fn autumn() -> Self {
        Self {
            kind: SeasonKind::Autumn,
            ..default()
        }
    }

    pub fn winter() -> Self {
        Self {
            kind: SeasonKind::Winter,
            ..default()
        }
    }
}

fn tick_timer(time: Res<Time>, mut season: ResMut<Season>) {
    if season.active && season.timer.tick(time.delta()).just_finished() {
        /* Run end of season game tick */

        /* Switch to next season */
    }
}

fn activate_season(mut season: ResMut<Season>) {
    if season.user_action_resource == 0 {
        season.active = true;
    }
}
