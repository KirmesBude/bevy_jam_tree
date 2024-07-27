//! Spawn the main level by triggering other observers.

use bevy::color::palettes::css::GREEN;
use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;

use bevy_ecs_tilemap::prelude::*;

use crate::game::assets::ImageAssets;
use crate::game::season::Season;
use crate::screen::Screen;

use super::tree::Tree;
use super::tree::OVERLAY_TEXTURE_INDEX_TREE;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin);
    app.observe(spawn_level);

    app.register_type::<(HighlightedTile, SelectedTile)>();
    app.init_resource::<HighlightedTile>();
    app.init_resource::<SelectedTile>();
    app.add_systems(
        Update,
        (
            reset_tile_color,
            update_highlighted_tile_color,
            update_selected_tile_color,
        )
            .chain()
            .run_if(in_state(Screen::Playing)),
    );
    app.add_systems(
        Update,
        (
            highlighted_tile_mouse,
            update_selected_tile_mouse,
            update_selected_tile_touch,
        )
            .chain()
            .run_if(in_state(Screen::Playing)),
    );

    app.add_systems(
        Update,
        update_ground_index.run_if(in_state(Screen::Playing)),
    );
}

#[derive(Debug, Component, Reflect)]
pub enum Ground {
    Normal,
    Nutrient,
}

impl Ground {
    pub fn name(&self) -> &'static str {
        match self {
            Ground::Normal => "Normal",
            Ground::Nutrient => "Nutrient",
        }
    }

    fn texture_index_offset(&self) -> u32 {
        match self {
            Ground::Normal => 0,
            Ground::Nutrient => 4,
        }
    }
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
    image_assets: Res<ImageAssets>,
) {
    // GroundLayer
    let texture_handle = image_assets.ground_tileset.clone_weak();

    let map_size = TilemapSize {
        x: MAP_SIZE,
        y: MAP_SIZE,
    };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    commands.entity(tilemap_id.0).with_children(|parent| {
        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let tile_pos = TilePos { x, y };

                let tile_entity = parent
                    .spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id,
                            texture_index: TileTextureIndex(0),
                            ..Default::default()
                        },
                        Ground::Normal,
                    ))
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    });

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
    let texture_handle = image_assets.tree_tileset.clone_weak();

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    commands.entity(tilemap_id.0).with_children(|parent| {
        for tile_pos in [
            TilePos::new(2, 3),
            TilePos::new(5, 4),
            TilePos::new(3, 3),
            TilePos::new(5, 6),
        ] {
            let tile_entity = parent
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id,
                        texture_index: TileTextureIndex(OVERLAY_TEXTURE_INDEX_TREE),
                        ..Default::default()
                    },
                    Tree::Immature,
                ))
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
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

    // Effect Layer
    let texture_handle = image_assets.effect_tileset.clone_weak();

    let tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    let tile_size = TilemapTileSize { x: 64.0, y: 112.0 };
    let grid_size = TilemapGridSize { x: 64.0, y: 32.0 };
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    commands.entity(tilemap_entity).insert((
        Name::new("EffectLayer"),
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
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 2.0),
            ..Default::default()
        },
        EffectLayer,
        StateScoped(Screen::Playing),
    ));
}

const HIGHLIGHT_COLOR: Color = bevy::prelude::Color::Srgba(RED);
const SELECTED_COLOR: Color = bevy::prelude::Color::Srgba(GREEN);

fn reset_tile_color(mut tile_colors: Query<&mut TileColor>) {
    /* Reset color */
    for mut tile_colors in &mut tile_colors {
        *tile_colors = TileColor::default();
    }
}

fn window_pos_to_tile_pos(
    cam_tuple: (&GlobalTransform, &Camera),
    window_pos: Vec2,
    tilemap_tuple: (&TilemapSize, &TilemapGridSize, &TilemapType, &Transform),
) -> Option<TilePos> {
    let (cam_t, cam) = cam_tuple;

    if let Some(cursor_world_position) = cam.viewport_to_world_2d(cam_t, window_pos) {
        let (map_size, grid_size, map_type, map_transform) = tilemap_tuple;

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
        TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
    } else {
        None
    }
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct HighlightedTile(pub Option<TilePos>);

fn highlighted_tile_mouse(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut highlighted_tile: ResMut<HighlightedTile>,
    tilemap_q: Query<(&TilemapSize, &TilemapGridSize, &TilemapType, &Transform), With<GroundLayer>>,
) {
    for cursor_moved in cursor_moved_events.read() {
        for cam_tuple in camera_q.iter() {
            highlighted_tile.0 =
                window_pos_to_tile_pos(cam_tuple, cursor_moved.position, tilemap_q.single());
        }
    }
}

fn update_highlighted_tile_color(
    highlighted_tile: Res<HighlightedTile>,
    ground_tile_storages: Query<&TileStorage, With<GroundLayer>>,
    mut tile_colors: Query<&mut TileColor>,
) {
    if let Some(tile_pos) = highlighted_tile.0 {
        if let Some(entity) = ground_tile_storages.single().get(&tile_pos) {
            if let Ok(mut tile_colors) = tile_colors.get_mut(entity) {
                *tile_colors = TileColor(HIGHLIGHT_COLOR);
            }
        }
    }
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct SelectedTile(pub Option<TilePos>);

fn update_selected_tile_mouse(
    highlighted_tile: Res<HighlightedTile>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut selected_tile: ResMut<SelectedTile>,
) {
    if mouse_input.just_pressed(MouseButton::Left) && highlighted_tile.0.is_some() {
        selected_tile.0 = highlighted_tile.0;
    }
}

fn update_selected_tile_touch(
    mut selected_tile: ResMut<SelectedTile>,
    mut touch_events: EventReader<TouchInput>,
    camera_q: Query<(&GlobalTransform, &Camera)>,
    tilemap_q: Query<(&TilemapSize, &TilemapGridSize, &TilemapType, &Transform), With<GroundLayer>>,
) {
    for touch_event in touch_events.read() {
        match touch_event.phase {
            bevy::input::touch::TouchPhase::Started
            | bevy::input::touch::TouchPhase::Moved
            | bevy::input::touch::TouchPhase::Ended => {
                for cam_tuple in camera_q.iter() {
                    selected_tile.0 =
                        window_pos_to_tile_pos(cam_tuple, touch_event.position, tilemap_q.single());
                }
            }
            bevy::input::touch::TouchPhase::Canceled => continue,
        }
    }
}

fn update_selected_tile_color(
    selected_tile: Res<SelectedTile>,
    ground_tile_storages: Query<&TileStorage, With<GroundLayer>>,
    mut tile_colors: Query<&mut TileColor>,
) {
    if let Some(tile_pos) = selected_tile.0 {
        if let Some(entity) = ground_tile_storages.single().get(&tile_pos) {
            if let Ok(mut tile_colors) = tile_colors.get_mut(entity) {
                *tile_colors = TileColor(SELECTED_COLOR);
            }
        }
    }
}

fn update_ground_index(
    mut ground_q: Query<(&mut TileTextureIndex, &Ground), Changed<Ground>>,
    season: Res<Season>,
) {
    for (mut texture_index, ground) in &mut ground_q {
        /* Actually do something interesting, like change texture index */
        let offset = ground.texture_index_offset();
        texture_index.0 = season.kind.texture_index() + offset;
    }
}
