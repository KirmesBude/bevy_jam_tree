use bevy::color::palettes::css::{
    BLACK, BLUE, BROWN, GREEN, RED, SLATE_GREY, WHITE, WHITE_SMOKE, YELLOW,
};
use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy_ecs_tilemap::tiles::TileStorage;

use crate::screen::Screen;
use crate::ui::palette::{BUTTON_HOVERED_BACKGROUND, BUTTON_PRESSED_BACKGROUND, NODE_BACKGROUND};
use crate::ui::prelude::{InteractionPalette, InteractionQuery};

use super::assets::{ImageAssets, UiAssets};
use super::season::state::{NextSeasonState, SeasonState};
use super::season::Season;
use super::spawn::level::{Ground, GroundLayer, SelectedTile, TreeLayer};
use super::spawn::tree::Tree;
use super::Score;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_game_ui);

    app.add_systems(
        Update,
        (
            update_selected_tree_image,
            update_selected_tree_text,
            update_selected_ground_image,
            update_selected_ground_text,
            update_season_header,
            update_season_clock,
            update_season_description,
            update_season_action,
            handle_season_action,
        )
            .run_if(in_state(Screen::Playing)),
    );
}

#[derive(Debug, Default, Event)]
pub struct SpawnGameUi;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct GameUi;

fn spawn_game_ui(_trigger: Trigger<SpawnGameUi>, mut commands: Commands) {
    commands
        .spawn((
            Name::new("Game UI Root"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
        ))
        .insert(StateScoped(Screen::Playing))
        .with_children(|parent| {
            left_ui_root(parent);
            middle_ui_root(parent);
            right_ui_root(parent);
        });
}

/// This shows information about the current selected tile
fn left_ui_root(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Name::new("Left Game UI"),
            NodeBundle {
                style: Style {
                    width: Percent(20.0),
                    height: Percent(100.0),
                    justify_content: JustifyContent::SpaceEvenly,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    left: Percent(0.0),
                    ..default()
                },
                background_color: BackgroundColor(RED.into()),
                ..default()
            },
        ))
        .with_children(|parent| {
            selected_tile_tree_ui(parent);
            selected_tile_ground_ui(parent);
        });
}

#[derive(Debug, Component, Reflect)]
pub struct SelectedTileTreeUi;

fn selected_tile_tree_ui(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Name::new("Selected Tile Tree UI"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(50.0),
                    justify_content: JustifyContent::SpaceEvenly,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    top: Percent(0.0),
                    ..default()
                },
                background_color: BackgroundColor(BLUE.into()),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(86.),
                        ..default()
                    },
                    background_color: BackgroundColor(SLATE_GREY.into()),
                    ..default()
                },
                TextureAtlas::default(),
                Outline::new(Val::Percent(2.0), Val::ZERO, GREEN.into()),
                SelectedTileTreeUi,
            ));
            parent.spawn((
                TextBundle::from_section(
                    "None",
                    TextStyle {
                        font_size: 40.0,
                        color: BLACK.into(),
                        ..default()
                    },
                )
                .with_background_color(WHITE_SMOKE.into()),
                Outline::new(Val::Percent(2.0), Val::ZERO, GREEN.into()),
                SelectedTileTreeUi,
            ));
        });
}

fn update_selected_tree_image(
    mut selected_tree_images: Query<(&mut UiImage, &mut TextureAtlas), With<SelectedTileTreeUi>>,
    season: Res<Season>,
    selected_tile: Res<SelectedTile>,
    tree_tile_storage: Query<&TileStorage, With<TreeLayer>>,
    trees: Query<&Tree>,
    image_assets: Res<ImageAssets>,
    ui_assets: Res<UiAssets>,
) {
    for (mut image, mut atlas) in &mut selected_tree_images {
        *image = UiImage::default();
        *atlas = TextureAtlas::default();

        // Do we have anything selected?
        if let Some(tile_pos) = selected_tile.0 {
            if let Some(entity) = tree_tile_storage.single().get(&tile_pos) {
                if let Ok(tree) = trees.get(entity) {
                    *image = UiImage {
                        texture: image_assets.tree_tileset.clone_weak(),
                        ..default()
                    };
                    *atlas = TextureAtlas {
                        layout: ui_assets.tree_layout.clone_weak(),
                        index: (tree.texture_index_offset() + season.kind.texture_index()) as usize,
                    }
                }
            }
        }
    }
}

fn update_selected_tree_text(
    mut selected_tree_texts: Query<&mut Text, With<SelectedTileTreeUi>>,
    selected_tile: Res<SelectedTile>,
    tree_tile_storage: Query<&TileStorage, With<TreeLayer>>,
    trees: Query<&Tree>,
) {
    for mut text in &mut selected_tree_texts {
        text.sections[0].value = String::from("None");

        // Do we have anything selected?
        if let Some(tile_pos) = selected_tile.0 {
            if let Some(entity) = tree_tile_storage.single().get(&tile_pos) {
                if let Ok(tree) = trees.get(entity) {
                    let text_value = tree.name().to_string();
                    text.sections[0].value.clone_from(&text_value);
                }
            }
        }
    }
}

#[derive(Debug, Component, Reflect)]
pub struct SelectedTileGroundUi;

fn selected_tile_ground_ui(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Name::new("Selected Tile Ground UI"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(50.0),
                    justify_content: JustifyContent::SpaceEvenly,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    bottom: Percent(0.0),
                    ..default()
                },
                background_color: BackgroundColor(BLUE.into()),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(86.),
                        ..default()
                    },
                    background_color: BackgroundColor(SLATE_GREY.into()),
                    ..default()
                },
                TextureAtlas::default(),
                Outline::new(Val::Percent(2.0), Val::ZERO, GREEN.into()),
                SelectedTileGroundUi,
            ));
            parent.spawn((
                TextBundle::from_section(
                    "None",
                    TextStyle {
                        font_size: 40.0,
                        color: BLACK.into(),
                        ..default()
                    },
                )
                .with_background_color(WHITE_SMOKE.into()),
                Outline::new(Val::Percent(2.0), Val::ZERO, GREEN.into()),
                SelectedTileGroundUi,
            ));
        });
}

fn update_selected_ground_image(
    mut selected_ground_images: Query<
        (&mut UiImage, &mut TextureAtlas),
        With<SelectedTileGroundUi>,
    >,
    season: Res<Season>,
    selected_tile: Res<SelectedTile>,
    ground_tile_storage: Query<&TileStorage, With<GroundLayer>>,
    ground_q: Query<&Ground>,
    image_assets: Res<ImageAssets>,
    ui_assets: Res<UiAssets>,
) {
    for (mut image, mut atlas) in &mut selected_ground_images {
        *image = UiImage::default();
        *atlas = TextureAtlas::default();

        // Do we have anything selected?
        if let Some(tile_pos) = selected_tile.0 {
            if let Some(entity) = ground_tile_storage.single().get(&tile_pos) {
                if let Ok(ground) = ground_q.get(entity) {
                    *image = UiImage {
                        texture: image_assets.ground_tileset.clone_weak(),
                        ..default()
                    };
                    *atlas = TextureAtlas {
                        layout: ui_assets.ground_layout.clone_weak(),
                        index: (ground.texture_index_offset() + season.kind.texture_index())
                            as usize,
                    }
                }
            }
        }
    }
}

fn update_selected_ground_text(
    mut selected_ground_texts: Query<&mut Text, With<SelectedTileGroundUi>>,
    selected_tile: Res<SelectedTile>,
    ground_tile_storage: Query<&TileStorage, With<GroundLayer>>,
    ground_q: Query<&Ground>,
) {
    for mut text in &mut selected_ground_texts {
        text.sections[0].value = String::from("None");

        // Do we have anything selected?
        if let Some(tile_pos) = selected_tile.0 {
            if let Some(entity) = ground_tile_storage.single().get(&tile_pos) {
                if let Ok(ground) = ground_q.get(entity) {
                    let text_value = ground.name().to_string();
                    text.sections[0].value.clone_from(&text_value);
                }
            }
        }
    }
}

fn middle_ui_root(parent: &mut ChildBuilder) {
    parent.spawn((
        Name::new("Middle Game UI"),
        NodeBundle {
            style: Style {
                width: Percent(60.0),
                height: Percent(100.0),
                justify_content: JustifyContent::SpaceEvenly,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
    ));
}

fn right_ui_root(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Name::new("Right Game UI"),
            NodeBundle {
                style: Style {
                    width: Percent(20.0),
                    height: Percent(100.0),
                    justify_content: JustifyContent::SpaceEvenly,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    right: Percent(0.0),
                    ..default()
                },
                background_color: BackgroundColor(BLUE.into()),
                ..default()
            },
        ))
        .with_children(|parent| {
            season_header_ui(parent);
            season_clock_ui(parent);
            season_description_ui(parent);
            season_action_ui(parent);
        });
}

#[derive(Debug, Component, Reflect)]
pub struct SeasonHeaderUi;

fn season_header_ui(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Name::new("Season Header UI"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(20.0),
                    top: Percent(0.0),
                    ..default()
                },
                background_color: BackgroundColor(YELLOW.into()),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Season Header",
                    TextStyle {
                        font_size: 40.0,
                        color: BLACK.into(),
                        ..default()
                    },
                ),
                SeasonHeaderUi,
            ));
        });
}

fn update_season_header(
    season: Res<Season>,
    mut season_header_texts: Query<&mut Text, With<SeasonHeaderUi>>,
) {
    for mut text in &mut season_header_texts {
        text.sections[0].value = format!("{}\nYear {}", season.kind.header(), season.year + 1);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct SeasonClockUi;

fn season_clock_ui(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Name::new("Season Clock UI"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(20.0),
                    ..default()
                },
                background_color: BackgroundColor(BROWN.into()),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Season Clock",
                    TextStyle {
                        font_size: 40.0,
                        color: BLACK.into(),
                        ..default()
                    },
                ),
                SeasonClockUi,
            ));
        });
}

fn update_season_clock(
    score: Res<Score>,
    mut season_clock_texts: Query<&mut Text, With<SeasonClockUi>>,
) {
    for mut text in &mut season_clock_texts {
        text.sections[0].value = format!("Score: {}", score.0);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct SeasonDescriptionUi;

fn season_description_ui(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Name::new("Season Description UI"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(40.0),
                    ..default()
                },
                background_color: BackgroundColor(WHITE.into()),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Season Description",
                    TextStyle {
                        font_size: 40.0,
                        color: BLACK.into(),
                        ..default()
                    },
                ),
                SeasonDescriptionUi,
            ));
        });
}

fn update_season_description(
    season: Res<Season>,
    mut season_description_texts: Query<&mut Text, With<SeasonDescriptionUi>>,
) {
    for mut text in &mut season_description_texts {
        text.sections[0].value = season.kind.description().into();
    }
}

#[derive(Debug, Component, Reflect)]
pub struct SeasonActionUi;

fn season_action_ui(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Name::new("Season Action UI"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(20.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(BROWN.into()),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Percent(95.0),
                            height: Percent(95.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(NODE_BACKGROUND),
                        ..default()
                    },
                    InteractionPalette {
                        none: NODE_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                    SeasonActionUi,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_sections([
                            TextSection {
                                value: "Action".into(),
                                style: TextStyle {
                                    font_size: 40.0,
                                    color: BLACK.into(),
                                    ..default()
                                },
                            },
                            TextSection {
                                value: "Season Resources".into(),
                                style: TextStyle {
                                    font_size: 40.0,
                                    color: BLACK.into(),
                                    ..default()
                                },
                            },
                        ]),
                        SeasonActionUi,
                    ));
                });
        });
}

fn update_season_action(
    season: Res<Season>,
    mut season_action_texts: Query<&mut Text, With<SeasonActionUi>>,
) {
    for mut text in &mut season_action_texts {
        if matches!(season.state, SeasonState::UserInput) {
            text.sections[0].value = String::from("Action");
            if season.user_action_resource > 0 {
                text.sections[1].value = String::from("");
            } else {
                text.sections[1].value = String::from("\nStart");
            }
        } else {
            text.sections[0].value = String::from("Simulating");
            text.sections[1].value = String::from("");
        }
    }
}

// TODO: This should be abstracted away better
fn handle_season_action(
    mut commands: Commands,
    mut button_query: InteractionQuery<&SeasonActionUi>,
    season: Res<Season>,
    mut next_season_state_events: EventWriter<NextSeasonState>,
) {
    for (interaction, _action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            if season.user_action_resource > 0 {
                season.kind.user_action(&mut commands);
            } else if matches!(season.state, SeasonState::UserInput) {
                next_season_state_events.send(NextSeasonState(season.state.next()));
            }
        }
    }
}
