use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use itertools::Itertools;

use crate::game::season::Season;
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

fn tree_game_of_life(
    time: Res<Time>,
    mut tick_time: Local<f32>,
    trees: Query<&TileStorage, With<TreeLayer>>,
    mut despawn_tree_events: EventWriter<DespawnTree>,
    mut spawn_tree_events: EventWriter<SpawnTree>,
) {
    *tick_time += time.delta_seconds();

    if *tick_time >= 5.0 {
        *tick_time = 0.0;

        let trees = trees.single();

        for x in 0..trees.size.x {
            for y in 0..trees.size.y {
                let tile_pos = TilePos { x, y };

                let neighbor_count =
                    Neighbors::get_square_neighboring_positions(&tile_pos, &trees.size, true)
                        .entities(trees)
                        .iter()
                        .count();

                match trees.get(&tile_pos) {
                    Some(_) => {
                        /* Live tree */
                        if !(2..=3).contains(&neighbor_count) {
                            despawn_tree_events.send(DespawnTree { tile_pos });
                        }
                    }
                    None => {
                        /* Dead tree */
                        if neighbor_count == 3 {
                            spawn_tree_events.send(SpawnTree {
                                tile_pos,
                                tree: Tree::Seedling,
                                use_resource: false,
                            });
                        }
                    }
                }
            }
        }
    }
}
