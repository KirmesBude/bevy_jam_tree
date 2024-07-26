use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileTextureIndex;
use state::SeasonState;

use crate::screen::Screen;

use super::spawn::tree::Tree;

pub mod logic;
pub mod state;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((state::plugin, logic::plugin));
    app.register_type::<(Season, SeasonKind)>();
    app.init_resource::<Season>();

    app.add_systems(
        Update,
        (handle_transition).run_if(in_state(Screen::Playing)),
    );
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
    pub state: SeasonState,
    pub kind: SeasonKind,
    pub user_action_resource: usize,
}

impl Default for Season {
    fn default() -> Self {
        Self {
            state: SeasonState::UserInput,
            kind: SeasonKind::Spring,
            user_action_resource: 4,
        }
    }
}

fn handle_transition(
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

            commands.entity(entity).remove::<SeasonTransition>();
        }
    }
}

#[derive(Debug, Reflect, Component)]
struct SeasonTransition {
    timer: Timer,
    season_kind: SeasonKind,
}
