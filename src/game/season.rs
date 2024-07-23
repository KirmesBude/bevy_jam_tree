use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

use crate::screen::Screen;

use super::RunGameLogic;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<(Season, SeasonKind)>();
    app.init_resource::<Season>();

    app.add_systems(
        Update,
        (
            activate_season,
            tick_season_timer,
            tick_transition_timer,
            advance_season,
        )
            .run_if(in_state(Screen::Playing)),
    );
    app.observe(transition_to_season);
}

#[derive(Clone, Copy, Debug, Reflect)]
pub enum SeasonKind {
    Spring, /* User action: Plant seeds; Passive effect: No trees die */
    Summer, /* User action: Place/combat(?) wildfires; Passive effects: Accelerated growth, random new trees */
    Autumn, /* User action: Wind direction?; Passive effect: Trees multiply */
    Winter, /* User action: AoE Heavy Snowfall(No trees are felled there); Passive effect: Trees are felled (mature/overmature) */
}

impl SeasonKind {
    pub fn next(&self) -> Self {
        match self {
            SeasonKind::Spring => SeasonKind::Summer,
            SeasonKind::Summer => SeasonKind::Autumn,
            SeasonKind::Autumn => SeasonKind::Winter,
            SeasonKind::Winter => SeasonKind::Spring,
        }
    }
}

#[derive(Debug, Reflect, Resource)]
#[reflect(Resource)]
pub struct Season {
    active: bool,
    timer: Timer,
    kind: SeasonKind,
    pub user_action_resource: usize,
}

impl Default for Season {
    fn default() -> Self {
        Self {
            active: false,
            timer: Timer::from_seconds(10.0, TimerMode::Once),
            kind: SeasonKind::Spring,
            user_action_resource: 1,
        }
    }
}

fn tick_season_timer(mut commands: Commands, time: Res<Time>, mut season: ResMut<Season>) {
    if season.active && season.timer.tick(time.delta()).just_finished() {
        /* Run end of season game tick */
        commands.trigger(RunGameLogic);

        /* Switch to next season */
        commands.trigger(TransitionToSeason {
            next_season: season.kind.next(),
        });
    }
}

fn activate_season(mut season: ResMut<Season>) {
    if season.user_action_resource == 0 {
        season.active = true;
    }
}

#[derive(Debug, Reflect, Event)]
struct TransitionToSeason {
    next_season: SeasonKind,
}

fn transition_to_season(
    trigger: Trigger<TransitionToSeason>,
    mut commands: Commands,
    tile_storages: Query<&TileStorage>,
) {
    /* We are adding the SeasonTransition Component with timer based on TilePos to each entity */
    for tile_storage in &tile_storages {
        for x in 0..tile_storage.size.x {
            for y in 0..tile_storage.size.y {
                if let Some(entity) = tile_storage.get(&TilePos { x, y }) {
                    commands.entity(entity).insert(SeasonTransition {
                        timer: Timer::from_seconds(0.1 + 0.1 * (x + y) as f32, TimerMode::Once),
                        season_kind: trigger.event().next_season,
                    });
                }
            }
        }
    }
}

fn advance_season(
    mut active: Local<bool>,
    transitions: Query<(), With<SeasonTransition>>,
    mut season: ResMut<Season>,
) {
    /* Season is finished when no more SeasonTransition components exist*/

    if *active {
        if transitions.is_empty() {
            *season = Season {
                kind: season.kind.next(),
                ..default()
            };
        }
        *active = false;
    } else {
        /* Need to have seen one first to activate */
        *active = !transitions.is_empty();
    }
}

fn tick_transition_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut transition_timers: Query<(Entity, &mut SeasonTransition)>,
) {
    for (entity, mut transition_timer) in &mut transition_timers {
        if transition_timer.timer.tick(time.delta()).just_finished() {
            /* Actually do something interesting, like change texture index */

            commands.entity(entity).remove::<SeasonTransition>();
        }
    }
}

#[derive(Debug, Reflect, Component)]
struct SeasonTransition {
    timer: Timer,
    season_kind: SeasonKind,
}
