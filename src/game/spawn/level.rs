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

const QUADRANT_SIDE_LENGTH: u32 = 4;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    //commands.trigger(SpawnPlayer);

    let texture_handle: Handle<Image> = asset_server.load("iso_color.png");

    // In total, there will be `(QUADRANT_SIDE_LENGTH * 2) * (QUADRANT_SIDE_LENGTH * 2)` tiles.
    let map_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH * 2,
        y: QUADRANT_SIDE_LENGTH * 2,
    };
    let quadrant_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH,
        y: QUADRANT_SIDE_LENGTH,
    };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap_rect(
        TileTextureIndex(0),
        TilePos { x: 0, y: 0 },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect(
        TileTextureIndex(1),
        TilePos {
            x: QUADRANT_SIDE_LENGTH,
            y: 0,
        },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect(
        TileTextureIndex(2),
        TilePos {
            x: 0,
            y: QUADRANT_SIDE_LENGTH,
        },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect(
        TileTextureIndex(3),
        TilePos {
            x: QUADRANT_SIDE_LENGTH,
            y: QUADRANT_SIDE_LENGTH,
        },
        quadrant_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 64.0, y: 32.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            map_type,
            render_settings: TilemapRenderSettings {
                // Map size is 12x12 so we'll have render chunks that are:
                // 12 tiles wide and 1 tile tall.
                render_chunk_size: UVec2::new(3, 1),
                y_sort: true,
            },
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        },
        StateScoped(Screen::Playing),
    ));

    // Overlay tilemap
    //
    let texture_handle: Handle<Image> = asset_server.load("tree.png");

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
                Overlay,
                Tree,
            ))
            .id();
        tile_storage.set(&tile_pos, tile_entity);
    });

    let tile_size = TilemapTileSize { x: 64.0, y: 96.0 };
    let grid_size = TilemapGridSize { x: 64.0, y: 32.0 };
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            map_type,
            render_settings: TilemapRenderSettings {
                // Map size is 12x12 so we'll have render chunks that are:
                // 12 tiles wide and 1 tile tall.
                render_chunk_size: UVec2::new(3, 1),
                y_sort: true,
            },
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
            ..Default::default()
        },
        Overlay,
        StateScoped(Screen::Playing),
    ));
}

#[derive(Debug, Default, Component)]
pub struct Overlay;
