use bevy::color::palettes::css::{BLACK, BLUE, BROWN, RED, WHITE, YELLOW};
use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy_ecs_tilemap::tiles::TileStorage;

use crate::screen::Screen;

use super::season::Season;
use super::spawn::level::{GroundLayer, TreeLayer};
use super::spawn::tree::{CursorPos, Tree};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_game_ui);

    app.add_systems(
        Update,
        (
            update_selected_tree,
            update_selected_ground,
            update_season_header,
            update_season_clock,
            update_season_description,
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
                TextBundle::from_section(
                    "Selected Tree",
                    TextStyle {
                        font_size: 40.0,
                        color: BLACK.into(),
                        ..default()
                    },
                ),
                SelectedTileTreeUi,
            ));
        });
}

fn update_selected_tree(
    mut selected_tree_texts: Query<&mut Text, With<SelectedTileTreeUi>>,
    season: Res<Season>,
    cursor: Res<CursorPos>,
    tree_tile_storage: Query<&TileStorage, With<TreeLayer>>,
    trees: Query<&Tree>,
) {
    // Do we have anything selected?
    let text_value = if let Some(tile_pos) = cursor.tile_pos {
        if let Some(entity) = tree_tile_storage.single().get(&tile_pos) {
            if let Ok(tree) = trees.get(entity) {
                format!("{} ({})", tree.name(), season.kind.header())
            } else {
                "None".into()
            }
        } else {
            "None".into()
        }
    } else {
        "None".into()
    };

    for mut text in &mut selected_tree_texts {
        text.sections[0].value.clone_from(&text_value);
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
                TextBundle::from_section(
                    "Selected Ground",
                    TextStyle {
                        font_size: 40.0,
                        color: BLACK.into(),
                        ..default()
                    },
                ),
                SelectedTileGroundUi,
            ));
        });
}

fn update_selected_ground(
    mut selected_ground_texts: Query<&mut Text, With<SelectedTileGroundUi>>,
    season: Res<Season>,
    cursor: Res<CursorPos>,
    ground_tile_storage: Query<&TileStorage, With<GroundLayer>>, /*, ground: Query<&Ground> */
) {
    // Do we have anything selected?
    let text_value = if let Some(tile_pos) = cursor.tile_pos {
        if let Some(_entity) = ground_tile_storage.single().get(&tile_pos) {
            /* if let Ok(ground) = grounds.get(entity) */
            {
                format!(
                    "{} ({})",
                    /* tree.name() */ "Ground",
                    season.kind.header()
                )
            }
        } else {
            "None".into()
        }
    } else {
        "None".into()
    };

    for mut text in &mut selected_ground_texts {
        text.sections[0].value.clone_from(&text_value);
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
        text.sections[0].value = season.kind.header().into();
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
    season: Res<Season>,
    mut season_clock_texts: Query<&mut Text, With<SeasonClockUi>>,
) {
    for mut text in &mut season_clock_texts {
        text.sections[0].value = format!("{:.2}", season.timer.remaining_secs());
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
                    height: Percent(60.0),
                    ..default()
                },
                background_color: BackgroundColor(WHITE.into()),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_sections([
                    TextSection {
                        value: "Season Description".into(),
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
        text.sections[1].value = format!("\nResources: {}", season.user_action_resource);
    }
}
