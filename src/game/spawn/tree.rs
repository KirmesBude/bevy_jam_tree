use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use itertools::Itertools;
use rand::Rng;

use crate::screen::Screen;

use super::level::Overlay;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Tree>();
    app.add_event::<SpawnTree>();
    app.add_systems(
        Update,
        (key_spawns, spawn_tree).run_if(in_state(Screen::Playing)),
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
        println!("{:?}", event);
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
                    ))
                    .id();
                overlay_storage.set(&tile_pos, tile_entity);
            });
        }
    }
}

fn key_spawns(input: Res<ButtonInput<KeyCode>>, mut spawn_tree_events: EventWriter<SpawnTree>) {
    if input.just_pressed(KeyCode::Space) {
        spawn_tree_events.send(SpawnTree(TilePos {
            x: rand::thread_rng().gen_range(0..=7),
            y: rand::thread_rng().gen_range(0..=7),
        }));
    }
}
