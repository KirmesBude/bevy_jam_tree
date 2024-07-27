//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use super::Screen;
use crate::{
    game::assets::{ImageAssets, SfxAssets, SoundtrackAssets, UiAssets},
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UiAssets>();
    app.add_systems(OnEnter(Screen::Loading), enter_loading);

    app.add_loading_state(
        LoadingState::new(Screen::Loading)
            .continue_to_state(Screen::Title)
            .load_collection::<ImageAssets>()
            .load_collection::<SfxAssets>()
            .load_collection::<SoundtrackAssets>(),
    );
}

fn enter_loading(
    mut commands: Commands,
    mut atlas: ResMut<Assets<TextureAtlasLayout>>,
    mut ui_asset: ResMut<UiAssets>,
) {
    let mut layout = TextureAtlasLayout::from_grid(
        UVec2::new(64, 48),
        4,
        2,
        Some(UVec2::new(0, 64)),
        Some(UVec2::new(0, 64)),
    );
    layout.size += UVec2::new(0, 64);
    let handle = atlas.add(layout);
    ui_asset.ground_layout = handle;

    let layout =
        TextureAtlasLayout::from_grid(UVec2::new(64, 96), 4, 4, Some(UVec2::new(0, 16)), None);
    let handle = atlas.add(layout);
    ui_asset.tree_layout = handle;

    commands
        .ui_root()
        .insert(StateScoped(Screen::Loading))
        .with_children(|children| {
            children.label("Loading...");
        });
}
