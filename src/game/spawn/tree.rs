use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use itertools::Itertools;

use crate::game::season::Season;
use crate::screen::Screen;

use super::level::{GroundLayer, TreeLayer};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Tree>();
    app.add_event::<SpawnTree>();
    app.add_event::<DespawnTree>();
    app.register_type::<HoveredTile>();
    app.init_resource::<HoveredTile>();
    app.add_systems(
        Update,
        (
            update_cursor_pos,
            update_hovered_tile_touch,
            spawn_tree_at_cursor,
            //tree_game_of_life,
            spawn_tree,
            despawn_tree,
            highlight_tile,
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
}

fn spawn_tree(
    mut commands: Commands,
    mut spawn_tree_events: EventReader<SpawnTree>,
    mut overlay_map: Query<(Entity, &mut TileStorage), With<TreeLayer>>,
    season: Res<Season>,
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
                            });
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct HoveredTile {
    pub tile_pos: Option<TilePos>,
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<HoveredTile>,
    tilemap_q: Query<(&TilemapSize, &TilemapGridSize, &TilemapType, &Transform), With<GroundLayer>>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Some(cursor_world_position) =
                cam.viewport_to_world_2d(cam_t, cursor_moved.position)
            {
                let (map_size, grid_size, map_type, map_transform) = tilemap_q.single();

                // We need to make sure that the cursor's world position is correct relative to the map
                // due to any map transformation.
                let cursor_in_map_pos = {
                    // Extend the cursor_pos vec2 by 0.0 and 1.0
                    let cursor_world_position = Vec4::from((cursor_world_position, 0.0, 1.0));
                    let cursor_in_map_pos =
                        map_transform.compute_matrix().inverse() * cursor_world_position;
                    cursor_in_map_pos.xy()
                };
                // TODO: Not sure why this is necessary
                let cursor_in_map_pos = cursor_in_map_pos + Vec2::new(0.0, grid_size.y / 2.0);
                // Once we have a world position we can transform it into a possible tile position.
                let tile_pos =
                    TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type);

                *cursor_pos = HoveredTile { tile_pos };
            }
        }
    }
}

fn update_hovered_tile_touch(
    mut hovered_tile: ResMut<HoveredTile>,
    mut touch_events: EventReader<TouchInput>,
    camera_q: Query<(&GlobalTransform, &Camera)>,
    tilemap_q: Query<(&TilemapSize, &TilemapGridSize, &TilemapType, &Transform), With<GroundLayer>>,
) {
    for touch_event in touch_events.read() {
        match touch_event.phase {
            bevy::input::touch::TouchPhase::Started
            | bevy::input::touch::TouchPhase::Moved
            | bevy::input::touch::TouchPhase::Ended => {
                // To get the mouse's world position, we have to transform its window position by
                // any transforms on the camera. This is done by projecting the cursor position into
                // camera space (world space).
                for (cam_t, cam) in camera_q.iter() {
                    if let Some(cursor_world_position) =
                        cam.viewport_to_world_2d(cam_t, touch_event.position)
                    {
                        let (map_size, grid_size, map_type, map_transform) = tilemap_q.single();

                        // We need to make sure that the cursor's world position is correct relative to the map
                        // due to any map transformation.
                        let cursor_in_map_pos = {
                            // Extend the cursor_pos vec2 by 0.0 and 1.0
                            let cursor_world_position =
                                Vec4::from((cursor_world_position, 0.0, 1.0));
                            let cursor_in_map_pos =
                                map_transform.compute_matrix().inverse() * cursor_world_position;
                            cursor_in_map_pos.xy()
                        };
                        // TODO: Not sure why this is necessary
                        let cursor_in_map_pos =
                            cursor_in_map_pos + Vec2::new(0.0, grid_size.y / 2.0);
                        // Once we have a world position we can transform it into a possible tile position.
                        let tile_pos = TilePos::from_world_pos(
                            &cursor_in_map_pos,
                            map_size,
                            grid_size,
                            map_type,
                        );

                        *hovered_tile = HoveredTile { tile_pos };
                    }
                }
            }
            bevy::input::touch::TouchPhase::Canceled => continue,
        }
    }
}

// This is where we check which tile the cursor is hovered over.
fn spawn_tree_at_cursor(
    mut spawn_tree_events: EventWriter<SpawnTree>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<HoveredTile>,
    mut season: ResMut<Season>,
) {
    if season.user_action_resource > 0 && mouse_input.just_pressed(MouseButton::Left) {
        if let Some(tile_pos) = cursor_pos.tile_pos {
            spawn_tree_events.send(SpawnTree {
                tile_pos,
                tree: Tree::Immature,
            });
            season.user_action_resource -= 1; /* TODO: I don't check whether it is occupied here, so may lose resource without placing a tree */
        }
    }
}

fn highlight_tile(
    cursor_pos: Res<HoveredTile>,
    tile_storages: Query<&TileStorage>,
    mut tile_colors: Query<&mut TileColor>,
) {
    /* Reset TileColor */
    for mut tile_color in &mut tile_colors {
        *tile_color = TileColor::default();
    }

    if let Some(tile_pos) = cursor_pos.tile_pos {
        // Highlight the relevant tile's label
        for tile_storage in &tile_storages {
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(mut tile_color) = tile_colors.get_mut(tile_entity) {
                    *tile_color = TileColor(RED.into());
                }
            }
        }
    }
}
