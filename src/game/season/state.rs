use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

use crate::screen::Screen;

use super::{
    logic::{SetupFelling, SetupGrowing, SetupOvercrowdDying, SetupSeedlingDying, TreeAction},
    Season, SeasonKind, SeasonTransition,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SeasonState>();
    app.add_event::<NextSeasonState>();

    app.observe(setup_user_input);
    app.observe(setup_simulation);
    app.observe(setup_transition);

    app.add_systems(
        Update,
        (to_transition, to_user_input, handle_next_season_state).run_if(in_state(Screen::Playing)),
    );
}

#[derive(Clone, Copy, Debug, Reflect)]
pub enum SeasonState {
    UserInput,
    Simulation,
    Transition,
}

impl SeasonState {
    pub fn next(&self) -> Self {
        match self {
            SeasonState::UserInput => SeasonState::Simulation,
            SeasonState::Simulation => SeasonState::Transition,
            SeasonState::Transition => SeasonState::UserInput,
        }
    }

    fn setup(&self, commands: &mut Commands, season_kind: SeasonKind) {
        match self {
            SeasonState::UserInput => commands.trigger(SetupUserInput(season_kind)),
            SeasonState::Simulation => commands.trigger(SetupSimulation(season_kind)),
            SeasonState::Transition => commands.trigger(SetupTransition(season_kind)),
        }
    }
}

#[derive(Debug, Event, Reflect)]
pub struct NextSeasonState(pub SeasonState);

fn handle_next_season_state(
    mut commands: Commands,
    mut next_season_state_events: EventReader<NextSeasonState>,
    mut season: ResMut<Season>,
) {
    for event in next_season_state_events.read() {
        season.state = event.0;

        // Do setup stuff
        season.state.setup(&mut commands, season.kind);
    }
}

// to_simulation is handled by ui

fn to_transition(
    season: Res<Season>,
    mut next_season_state_events: EventWriter<NextSeasonState>,
    tree_action_q: Query<&TreeAction>,
    mut looking: Local<bool>,
) {
    if matches!(season.state, SeasonState::Simulation) {
        if *looking && tree_action_q.is_empty() {
            next_season_state_events.send(NextSeasonState(season.state.next()));
            *looking = false;
        } else if !tree_action_q.is_empty() {
            *looking = true;
        }
    }
}

fn to_user_input(
    season: Res<Season>,
    mut next_season_state_events: EventWriter<NextSeasonState>,
    season_transition_q: Query<&SeasonTransition>,
    mut looking: Local<bool>,
) {
    if matches!(season.state, SeasonState::Transition) {
        if *looking && season_transition_q.is_empty() {
            next_season_state_events.send(NextSeasonState(season.state.next()));
            *looking = false;
        } else if !season_transition_q.is_empty() {
            *looking = true;
        }
    }
}

#[derive(Debug, Event)]
struct SetupUserInput(SeasonKind);

fn setup_user_input(trigger: Trigger<SetupUserInput>, mut season: ResMut<Season>) {
    season.kind = trigger.event().0.next();
    season.user_action_resource = 4; // TODO: Needs to be different per season, probably
}

#[derive(Debug, Event)]
struct SetupSimulation(SeasonKind);

fn setup_simulation(trigger: Trigger<SetupSimulation>, mut commands: Commands) {
    match trigger.event().0 {
        SeasonKind::Spring => {
            commands.trigger(SetupGrowing);
            commands.trigger(SetupOvercrowdDying);
        }
        SeasonKind::Summer => {
            commands.trigger(SetupGrowing);
        }
        SeasonKind::Autumn => {
            commands.trigger(SetupGrowing);
            commands.trigger(SetupOvercrowdDying);
        }
        SeasonKind::Winter => {
            commands.trigger(SetupSeedlingDying);
            commands.trigger(SetupFelling);
        }
    }
}

#[derive(Debug, Event)]
struct SetupTransition(SeasonKind);

fn setup_transition(
    trigger: Trigger<SetupTransition>,
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
                        season_kind: trigger.event().0.next(),
                    });
                }
            }
        }
    }
}
