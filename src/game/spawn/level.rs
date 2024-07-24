//! Spawn the main level by triggering other observers.

use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;

use bevy_ecs_tilemap::prelude::*;

use crate::screen::Screen;

use super::tree::Tree;
use super::tree::OVERLAY_TEXTURE_INDEX_TREE;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin);
    app.observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Debug, Default, Component, Reflect)]
pub struct GroundLayer;

#[derive(Debug, Default, Component, Reflect)]
pub struct TreeLayer;

#[derive(Debug, Default, Component, Reflect)]
pub struct EffectLayer;

const MAP_SIZE: u32 = 8;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // GroundLayer
    let texture_handle: Handle<Image> = asset_server.load("images/ground_tileset.png");

    let map_size = TilemapSize {
        x: MAP_SIZE,
        y: MAP_SIZE,
    };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap_rect(
        TileTextureIndex(0),
        TilePos { x: 0, y: 0 },
        map_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 64.0, y: 112.0 };
    let grid_size = TilemapGridSize { x: 64.0, y: 32.0 };
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    commands.entity(tilemap_entity).insert((
        Name::new("GroundLayer"),
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            map_type,
            render_settings: TilemapRenderSettings {
                render_chunk_size: UVec2::new(MAP_SIZE, 1),
                y_sort: true,
            },
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        },
        StateScoped(Screen::Playing),
        GroundLayer,
    ));

    // Tree Layer
    let texture_handle: Handle<Image> = asset_server.load("images/tree_tileset.png");

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    commands.entity(tilemap_id.0).with_children(|parent| {
        let tile_pos = TilePos { x: 4, y: 4 };

        let tile_entity = parent
            .spawn((
                TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    texture_index: TileTextureIndex(OVERLAY_TEXTURE_INDEX_TREE),
                    ..Default::default()
                },
                Tree::default(),
            ))
            .id();
        tile_storage.set(&tile_pos, tile_entity);
    });

    let tile_size = TilemapTileSize { x: 64.0, y: 112.0 };
    let grid_size = TilemapGridSize { x: 64.0, y: 32.0 };
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    commands.entity(tilemap_entity).insert((
        Name::new("TreeLayer"),
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            map_type,
            render_settings: TilemapRenderSettings {
                render_chunk_size: UVec2::new(MAP_SIZE, 1),
                y_sort: true,
            },
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
            ..Default::default()
        },
        TreeLayer,
        StateScoped(Screen::Playing),
    ));
}
