use bevy::prelude::*;
use bevy_ecs_tilemap::{
    helpers::square_grid::{
        neighbors::{Neighbors, SquareDirection},
        SquarePos,
    },
    tiles::{TileStorage, TileTextureIndex},
};
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;
use logic::TreeAction;
use state::SeasonState;

use crate::screen::Screen;

use super::spawn::{
    level::{Ground, SelectedTile, TreeLayer},
    tree::{SpawnTree, Tree},
};

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

    app.observe(spring_user_action);
    app.observe(summer_user_action);
    app.observe(autumn_user_action);
    app.observe(winter_user_action);
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
            SeasonKind::Spring => "Place 4 seedlings in spring and watch them grow.",
            SeasonKind::Summer => "No trees ever die in summer...\nBut you have to set fire to a tree. Be careful, it spreads quickly. Leaves behind nutrient soil for mature and overmature trees.",
            SeasonKind::Autumn => "Place a good gust on any tree to let it drop its seeds in a cross pattern one tile away.",
            SeasonKind::Winter => "Any seedlings will succumb to the cold. Any mature and overmature trees are taken by the local folk for points.\nDirect snow storms to selected trees to keep them around for another time.",
        }
    }

    pub fn user_action(&self, commands: &mut Commands) {
        match self {
            SeasonKind::Spring => commands.trigger(SpringUserAction),
            SeasonKind::Summer => commands.trigger(SummerUserAction),
            SeasonKind::Autumn => commands.trigger(AutumnUserAction),
            SeasonKind::Winter => commands.trigger(WinterUserAction),
        }
    }
}

#[derive(Debug, Reflect, Resource)]
#[reflect(Resource)]
pub struct Season {
    pub year: u32,
    pub state: SeasonState,
    pub kind: SeasonKind,
    pub user_action_resource: usize,
}

impl Default for Season {
    fn default() -> Self {
        Self {
            year: 0,
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
        Option<&Ground>,
    )>,
) {
    for (entity, mut season_transition, mut texture_index, tree, ground) in &mut transition_timers {
        if season_transition.timer.tick(time.delta()).just_finished() {
            /* Actually do something interesting, like change texture index */
            let offset = if let Some(tree) = tree {
                tree.texture_index_offset()
            } else if let Some(ground) = ground {
                ground.texture_index_offset()
            } else {
                0
            };
            texture_index.0 = season_transition.season_kind.texture_index() + offset;

            commands.entity(entity).remove::<SeasonTransition>();
            commands.entity(entity).remove::<BadWeather>();
        }
    }
}

#[derive(Debug, Reflect, Component)]
struct SeasonTransition {
    timer: Timer,
    season_kind: SeasonKind,
}

#[derive(Debug, Event)]
pub struct SpringUserAction;

fn spring_user_action(
    _trigger: Trigger<SpringUserAction>,
    mut selected_tile: ResMut<SelectedTile>,
    mut spawn_tree_events: EventWriter<SpawnTree>,
) {
    if let Some(tile_pos) = selected_tile.0 {
        spawn_tree_events.send(SpawnTree {
            tile_pos,
            tree: Tree::Seedling,
            use_resource: true,
        });

        selected_tile.0 = None;
    }
}

#[derive(Debug, Event)]
pub struct SummerUserAction;

fn summer_user_action(
    _trigger: Trigger<SummerUserAction>,
    mut selected_tile: ResMut<SelectedTile>,
    mut season: ResMut<Season>,
    mut commands: Commands,
    tree_tile_storage_q: Query<&TileStorage, With<TreeLayer>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    if let Some(tile_pos) = selected_tile.0 {
        let tile_storage = tree_tile_storage_q.single();
        if let Some(entity) = tile_storage.checked_get(&tile_pos) {
            commands
                .entity(entity)
                .insert(TreeAction::burning(&mut rng));

            season.user_action_resource = 0;
            selected_tile.0 = None;
        }
    }
}

#[derive(Debug, Event)]
pub struct AutumnUserAction;

fn autumn_user_action(
    _trigger: Trigger<AutumnUserAction>,
    mut season: ResMut<Season>,
    mut selected_tile: ResMut<SelectedTile>,
    mut spawn_tree_events: EventWriter<SpawnTree>,
    tree_q: Query<&Tree>,
    tree_tile_storage_q: Query<&TileStorage, With<TreeLayer>>,
) {
    if let Some(tile_pos) = selected_tile.0 {
        let tile_storage = tree_tile_storage_q.single();
        if let Some(entity) = tile_storage.checked_get(&tile_pos) {
            if let Ok(tree) = tree_q.get(entity) {
                if matches!(tree, Tree::Mature | Tree::Overmature) {
                    let square_pos = SquarePos::from(&tile_pos);
                    let f = |direction: SquareDirection| {
                        if direction.is_cardinal() {
                            square_pos
                                .offset(&direction)
                                .offset(&direction)
                                .as_tile_pos(&tile_storage.size)
                        } else {
                            None
                        }
                    };

                    Neighbors::from_directional_closure(f)
                        .iter()
                        .for_each(|tile_pos| {
                            spawn_tree_events.send(SpawnTree {
                                tile_pos: *tile_pos,
                                tree: Tree::Seedling,
                                use_resource: false,
                            });
                        });

                    season.user_action_resource = 0;
                    selected_tile.0 = None;
                }
            }
        }
    }
}

#[derive(Debug, Event)]
pub struct WinterUserAction;

fn winter_user_action(
    _trigger: Trigger<WinterUserAction>,
    mut season: ResMut<Season>,
    mut selected_tile: ResMut<SelectedTile>,
    tree_tile_storage_q: Query<&TileStorage, With<TreeLayer>>,
    mut commands: Commands,
) {
    if let Some(tile_pos) = selected_tile.0 {
        let tile_storage = tree_tile_storage_q.single();

        if let Some(entity) = tile_storage.get(&tile_pos) {
            commands.entity(entity).insert(BadWeather);
            season.user_action_resource -= 2;
            selected_tile.0 = None;
        }
    }
}

#[derive(Debug, Default, Component, Reflect)]
pub struct BadWeather;
