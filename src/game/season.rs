use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage, TileTextureIndex};

use crate::screen::Screen;

use super::spawn::tree::Tree;
use super::RunGameLogic;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<(Season, SeasonKind)>();
    app.init_resource::<Season>();

    app.add_systems(
        Update,
        (tick_season_timer, tick_transition_timer, advance_season)
            .run_if(in_state(Screen::Playing)),
    );
    app.observe(transition_to_season);
}

#[derive(Clone, Copy, Debug, Reflect)]
pub enum SeasonKind {
    Spring,
    Summer,
    Autumn,
    Winter,
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

    pub fn texture_index(&self) -> u32 {
        match self {
            SeasonKind::Spring => 0,
            SeasonKind::Summer => 1,
            SeasonKind::Autumn => 2,
            SeasonKind::Winter => 3,
        }
    }

    pub fn header(&self) -> &'static str {
        match self {
            SeasonKind::Spring => "Spring",
            SeasonKind::Summer => "Summer",
            SeasonKind::Autumn => "Autumn",
            SeasonKind::Winter => "Winter",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SeasonKind::Spring => "User action: Plant seeds; Passive effect: No trees die",
            SeasonKind::Summer => "User action: Place/combat(?) wildfires; Passive effects: Accelerated growth, random new trees",
            SeasonKind::Autumn => "User action: Wind direction?; Passive effect: Trees multiply",
            SeasonKind::Winter => "User action: AoE Heavy Snowfall(No trees are felled there); Passive effect: Trees are felled (mature/overmature)",
        }
    }
}

#[derive(Debug, Reflect, Resource)]
#[reflect(Resource)]
pub struct Season {
    pub active: bool,
    pub timer: Timer,
    pub kind: SeasonKind,
    pub user_action_resource: usize,
}

impl Default for Season {
    fn default() -> Self {
        Self {
            active: false,
            timer: Timer::from_seconds(10.0, TimerMode::Once),
            kind: SeasonKind::Spring,
            user_action_resource: 4,
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
            *active = false;
        }
    } else {
        /* Need to have seen one first to activate */
        *active = !transitions.is_empty();
    }
}

fn tick_transition_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut transition_timers: Query<(
        Entity,
        &mut SeasonTransition,
        &mut TileTextureIndex,
        Option<&Tree>,
    )>,
) {
    for (entity, mut season_transition, mut texture_index, tree) in &mut transition_timers {
        if season_transition.timer.tick(time.delta()).just_finished() {
            /* Actually do something interesting, like change texture index */
            let offset = if let Some(tree) = tree {
                tree.texture_index_offset()
            } else {
                0
            };
            texture_index.0 = season_transition.season_kind.texture_index() + offset;
            println!(
                "{:?}, {:?}, {:?}",
                tree,
                offset,
                season_transition.season_kind.texture_index()
            );

            commands.entity(entity).remove::<SeasonTransition>();
        }
    }
}

#[derive(Debug, Reflect, Component)]
struct SeasonTransition {
    timer: Timer,
    season_kind: SeasonKind,
}
