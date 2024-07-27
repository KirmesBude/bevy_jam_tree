use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "images/ground_tileset.png")]
    #[asset(image(sampler = nearest))]
    pub ground_tileset: Handle<Image>,
    #[asset(path = "images/tree_tileset.png")]
    #[asset(image(sampler = nearest))]
    pub tree_tileset: Handle<Image>,
    #[asset(path = "images/effect_tileset.png")]
    #[asset(image(sampler = nearest))]
    pub effect_tileset: Handle<Image>,
}

#[derive(AssetCollection, Resource, Default)]
pub struct UiAssets {
    #[asset(texture_atlas_layout(
        tile_size_x = 64,
        tile_size_y = 48,
        columns = 4,
        rows = 2,
        padding_y = 64,
        offset_y = 64
    ))] //TODO: Bug in bevy_asset_loader?
    pub ground_layout: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(
        tile_size_x = 64,
        tile_size_y = 96,
        columns = 4,
        rows = 4,
        padding_y = 16
    ))]
    pub tree_layout: Handle<TextureAtlasLayout>,
}

#[derive(AssetCollection, Resource)]
pub struct SfxAssets {
    #[asset(path = "audio/sfx/button_hover.ogg")]
    pub button_hover: Handle<AudioSource>,
    #[asset(path = "audio/sfx/button_press.ogg")]
    pub button_press: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct SoundtrackAssets {
    #[asset(path = "audio/soundtracks/Vindsvept - Crystal Forest.ogg")]
    pub title: Handle<AudioSource>,
    #[asset(path = "audio/soundtracks/Vindsvept - Fall of the Leaf.ogg")]
    pub credits: Handle<AudioSource>,
    #[asset(path = "audio/soundtracks/Vindsvept - Woodland Lullaby.ogg")]
    pub gameplay: Handle<AudioSource>,
    #[asset(path = "audio/soundtracks/Vindsvept - Season Unending.ogg")]
    pub _gameplay2: Handle<AudioSource>,
}
