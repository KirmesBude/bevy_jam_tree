use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use itertools::Itertools;
use rand::Rng;

use crate::game::season::{Season, StartSeason};
use crate::screen::Screen;

use super::level::TreeLayer;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Tree>();
    app.add_event::<SpawnTree>();
    app.add_event::<DespawnTree>();
    app.add_systems(
        Update,
        (
            //tree_game_of_life,
            spawn_tree,
            despawn_tree,
        )
            .run_if(in_state(Screen::Playing)),
    );
    app.add_systems(
        Update,
        (update_tree_index, grow_action).run_if(in_state(Screen::Playing)),
    );

    app.observe(tree_logic);
}

pub const OVERLAY_TEXTURE_INDEX_TREE: u32 = 0;

/// Advances a state each gametick?
/// Overmature trees will die on next gametick (under specific circumstances?)
#[derive(Clone, Copy, Default, Debug, Component, Reflect, PartialEq, Eq, Hash)]
pub enum Tree {
    #[default]
    Seedling,
    Immature,
    Mature,
    Overmature,
}

impl Tree {
    pub const fn score(&self) -> usize {
        match self {
            Tree::Seedling => 0,
            Tree::Immature => 2,
            Tree::Mature => 5,
            Tree::Overmature => 6,
        }
    }

    pub const fn texture_index_offset(&self) -> u32 {
        match self {
            Tree::Seedling => 0,
            Tree::Immature => 4,
            Tree::Mature => 8,
            Tree::Overmature => 12,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tree::Seedling => "Seedling",
            Tree::Immature => "Immature",
            Tree::Mature => "Mature",
            Tree::Overmature => "Overmature",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Tree::Seedling => Some(Tree::Immature),
            Tree::Immature => Some(Tree::Mature),
            Tree::Mature => Some(Tree::Overmature),
            Tree::Overmature => None,
        }
    }

    pub fn level(&self) -> u32 {
        match self {
            Tree::Seedling => 1,
            Tree::Immature => 1,
            Tree::Mature => 1,
            Tree::Overmature => 2,
        }
    }
}

#[derive(Debug, Event, PartialEq, Eq, Hash)]
pub struct SpawnTree {
    pub tile_pos: TilePos,
    pub tree: Tree,
    pub use_resource: bool,
}

fn spawn_tree(
    mut commands: Commands,
    mut spawn_tree_events: EventReader<SpawnTree>,
    mut overlay_map: Query<(Entity, &mut TileStorage), With<TreeLayer>>,
    mut season: ResMut<Season>,
) {
    let (overlay_entity, mut overlay_storage) = overlay_map.single_mut();
    for event in spawn_tree_events.read().unique() {
        let tile_pos = event.tile_pos;
        if overlay_storage.checked_get(&tile_pos).is_none() {
            let tilemap_id = TilemapId(overlay_entity);
            commands.entity(overlay_entity).with_children(|parent| {
                let tile_entity = parent
                    .spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id,
                            texture_index: TileTextureIndex(
                                season.kind.texture_index() + event.tree.texture_index_offset(),
                            ),
                            ..Default::default()
                        },
                        event.tree,
                    ))
                    .id();
                overlay_storage.set(&tile_pos, tile_entity);

                if event.use_resource && season.user_action_resource > 0 {
                    season.user_action_resource -= 1;
                }
            });
        }
    }
}

fn despawn_tree(
    mut commands: Commands,
    mut despawn_tree_events: EventReader<DespawnTree>,
    mut overlay_map: Query<&mut TileStorage, With<TreeLayer>>,
) {
    let mut overlay_storage = overlay_map.single_mut();
    for event in despawn_tree_events.read().unique() {
        let tile_pos = event.tile_pos;

        if let Some(entity) = overlay_storage.checked_get(&tile_pos) {
            commands.entity(entity).despawn_recursive();
            overlay_storage.remove(&tile_pos);
        }
    }
}

#[derive(Debug, Event, PartialEq, Eq, Hash)]
pub struct DespawnTree {
    pub tile_pos: TilePos,
}

/// Goes over the board and assign TreeActions to each tree
fn tree_logic(
    _trigger: Trigger<StartSeason>,
    mut commands: Commands,
    tree_tile_storage_q: Query<&TileStorage, With<TreeLayer>>,
    tree_q: Query<(Entity, &Tree, &TilePos)>,
) {
    let tree_tile_storage = tree_tile_storage_q.single();

    for (tree_entity, tree, tile_pos) in &tree_q {
        let neighbor_level =
            Neighbors::get_square_neighboring_positions(&tile_pos, &tree_tile_storage.size, true)
                .entities(tree_tile_storage)
                .iter()
                .map(|entity| {
                    if let Ok((_, tree, _)) = tree_q.get(*entity) {
                        tree.level()
                    } else {
                        0
                    }
                })
                .sum();

        match neighbor_level {
            0..=2 => {
                /* Grow */
                commands.entity(tree_entity).insert(GrowAction::default());
            }
            3..=4 => { /* Do not grow */ }
            _ => {
                /* Die */
                commands.entity(tree_entity).insert(DieAction::default());
            }
        }
    }
}

fn update_tree_index(
    mut tree_q: Query<(&mut TileTextureIndex, &Tree), Changed<Tree>>,
    season: Res<Season>,
) {
    for (mut texture_index, tree) in &mut tree_q {
        /* Actually do something interesting, like change texture index */
        let offset = tree.texture_index_offset();
        texture_index.0 = season.kind.texture_index() + offset;
    }
}

/// Tree Actions

#[derive(Debug, Default, Component, Reflect)]
pub struct GrowAction(Timer);

impl GrowAction {
    pub fn new() -> Self {
        Self(Timer::from_seconds(
            rand::thread_rng().gen_range(2.5..7.5),
            TimerMode::Once,
        ))
    }
}

fn grow_action(
    mut commands: Commands,
    time: Res<Time>,
    mut grow_tree_q: Query<(Entity, &mut Tree, &mut GrowAction)>,
) {
    for (entity, mut tree, mut grow_action) in &mut grow_tree_q {
        if grow_action.0.tick(time.delta()).just_finished() {
            if let Some(next) = tree.next() {
                *tree = next;
            }

            commands.entity(entity).remove::<GrowAction>();
        }
    }
}

#[derive(Debug, Default, Component, Reflect)]
pub struct MultiplyAction(Timer);

impl MultiplyAction {
    pub fn new() -> Self {
        Self(Timer::from_seconds(
            rand::thread_rng().gen_range(2.5..7.5),
            TimerMode::Once,
        ))
    }
}

#[derive(Debug, Default, Component, Reflect)]
pub struct DieAction(Timer);

impl DieAction {
    pub fn new() -> Self {
        Self(Timer::from_seconds(
            rand::thread_rng().gen_range(2.5..7.5),
            TimerMode::Once,
        ))
    }
}

#[derive(Debug, Default, Component, Reflect)]
pub struct FellAction(Timer);

impl FellAction {
    pub fn new() -> Self {
        Self(Timer::from_seconds(
            rand::thread_rng().gen_range(2.5..7.5),
            TimerMode::Once,
        ))
    }
}
