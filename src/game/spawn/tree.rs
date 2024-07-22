use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use itertools::Itertools;

use crate::screen::Screen;

use super::level::Overlay;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Tree>();
    app.add_event::<SpawnTree>();
    app.add_event::<DespawnTree>();
    app.init_resource::<CursorPos>();
    app.add_systems(
        Update,
        (
            update_cursor_pos,
            spawn_tree_at_cursor,
            tree_game_of_life,
            spawn_tree,
            despawn_tree,
        )
            .run_if(in_state(Screen::Playing)),
    );
}

pub const OVERLAY_TEXTURE_INDEX_TREE: u32 = 0;

#[derive(Default, Debug, Component, Reflect)]
pub struct Tree;

#[derive(Debug, Event, PartialEq, Eq, Hash)]
pub struct SpawnTree(pub TilePos);

fn spawn_tree(
    mut commands: Commands,
    mut spawn_tree_events: EventReader<SpawnTree>,
    mut overlay_map: Query<(Entity, &mut TileStorage), With<Overlay>>,
) {
    let (overlay_entity, mut overlay_storage) = overlay_map.single_mut();
    for event in spawn_tree_events.read().unique() {
        let tile_pos = event.0;
        if overlay_storage.checked_get(&tile_pos).is_none() {
            let tilemap_id = TilemapId(overlay_entity);
            commands.entity(overlay_entity).with_children(|parent| {
                let tile_entity = parent
                    .spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id,
                            texture_index: TileTextureIndex(OVERLAY_TEXTURE_INDEX_TREE),
                            ..Default::default()
                        },
                        Overlay,
                        Tree,
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
    mut overlay_map: Query<&mut TileStorage, With<Overlay>>,
) {
    let mut overlay_storage = overlay_map.single_mut();
    for event in despawn_tree_events.read().unique() {
        let tile_pos = event.0;

        if let Some(entity) = overlay_storage.checked_get(&tile_pos) {
            commands.entity(entity).despawn_recursive();
            overlay_storage.remove(&tile_pos);
        }
    }
}

#[derive(Debug, Event, PartialEq, Eq, Hash)]
pub struct DespawnTree(pub TilePos);

fn tree_game_of_life(
    time: Res<Time>,
    mut tick_time: Local<f32>,
    trees: Query<&TileStorage, With<Overlay>>,
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
                            despawn_tree_events.send(DespawnTree(tile_pos));
                        }
                    }
                    None => {
                        /* Dead tree */
                        if neighbor_count == 3 {
                            spawn_tree_events.send(SpawnTree(tile_pos));
                        }
                    }
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct CursorPos(Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
}

// This is where we check which tile the cursor is hovered over.
fn spawn_tree_at_cursor(
    mut spawn_tree_events: EventWriter<SpawnTree>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(&TilemapSize, &TilemapGridSize, &TilemapType, &Transform), With<Overlay>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let (map_size, grid_size, map_type, map_transform) = tilemap_q.single();

        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec2 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            spawn_tree_events.send(SpawnTree(tile_pos));
        }
    }
}
