use bevy::{ecs::entity::EntityHashMap, prelude::*};
use bevy_ecs_tilemap::{
    helpers::square_grid::neighbors::Neighbors,
    map::TilemapId,
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;
use rand_core::RngCore;

use crate::{
    game::{
        spawn::{
            level::{EffectLayer, Ground, GroundLayer, TreeLayer},
            tree::{grow_logic, overcrowd_dying_logic, DespawnTree, Tree},
        },
        Score,
    },
    screen::Screen,
};

use super::{state::SeasonState, BadWeather, Season};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<TreeAction>();

    app.observe(setup_growing);
    app.observe(setup_overcrowd_dying);
    app.observe(setup_seedling_dying);
    app.observe(setup_felling);

    app.observe(grow);
    app.observe(die);
    app.observe(burn);
    app.observe(fell);

    app.add_systems(
        Update,
        (
            handle_tree_action,
            remove_effects,
            handle_effects,
            handle_bad_weather,
        )
            .chain()
            .run_if(in_state(Screen::Playing)),
    );
}

// Spring, Summer, Autumn
#[derive(Debug, Event)]
pub struct SetupGrowing;

fn setup_growing(
    _trigger: Trigger<SetupGrowing>,
    mut commands: Commands,
    tree_tile_storage_q: Query<&TileStorage, With<TreeLayer>>,
    tree_q: Query<(Entity, &Tree, &TilePos), Without<TreeAction>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    grow_logic(
        &mut commands,
        tree_tile_storage_q.single(),
        tree_q,
        &mut rng,
    );
}

// Spring, Autumn
#[derive(Debug, Event)]
pub struct SetupOvercrowdDying;

fn setup_overcrowd_dying(
    _trigger: Trigger<SetupOvercrowdDying>,
    mut commands: Commands,
    tree_tile_storage_q: Query<&TileStorage, With<TreeLayer>>,
    tree_q: Query<(Entity, &Tree, &TilePos), Without<TreeAction>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    overcrowd_dying_logic(
        &mut commands,
        tree_tile_storage_q.single(),
        tree_q,
        &mut rng,
    );
}

// Winter
#[derive(Debug, Event)]
pub struct SetupSeedlingDying;

fn setup_seedling_dying(
    _trigger: Trigger<SetupSeedlingDying>,
    mut commands: Commands,
    tree_q: Query<(Entity, &Tree)>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    for (entity, tree) in &tree_q {
        if matches!(tree, Tree::Seedling) {
            commands.entity(entity).insert(TreeAction::dying(&mut rng));
        }
    }
}

#[derive(Debug, Event)]
pub struct SetupFelling;

fn setup_felling(
    _trigger: Trigger<SetupFelling>,
    mut commands: Commands,
    tree_q: Query<(Entity, &Tree), Without<BadWeather>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    for (entity, tree) in &tree_q {
        if matches!(tree, Tree::Mature | Tree::Overmature) {
            commands
                .entity(entity)
                .insert(TreeAction::felling(&mut rng));
        }
    }
}

#[derive(Debug, Clone, Copy, Reflect)]
enum TreeActionKind {
    Growing,
    Dying,
    Burning,
    Felling,
}

impl TreeActionKind {
    fn trigger(&self, commands: &mut Commands, entity: Entity) {
        match self {
            TreeActionKind::Growing => commands.trigger(Grow(entity)),
            TreeActionKind::Dying => commands.trigger(Die(entity)),
            TreeActionKind::Burning => commands.trigger(Burn(entity)),
            TreeActionKind::Felling => commands.trigger(Fell(entity)),
        }
    }

    fn effect_texture_index(&self) -> u32 {
        match self {
            TreeActionKind::Growing => 0,
            TreeActionKind::Dying => 1,
            TreeActionKind::Burning => 2,
            TreeActionKind::Felling => 3,
        }
    }
}

#[derive(Debug, Component, Reflect)]
pub struct TreeAction {
    kind: TreeActionKind,
    timer: Timer,
}

impl TreeAction {
    pub fn growing(rng: &mut GlobalEntropy<WyRand>) -> Self {
        let duration = (rng.next_u32() % 30) as f32 * 0.1 + 1.0;
        Self {
            kind: TreeActionKind::Growing,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn dying(rng: &mut GlobalEntropy<WyRand>) -> Self {
        let duration = (rng.next_u32() % 30) as f32 * 0.1 + 1.0;
        Self {
            kind: TreeActionKind::Dying,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn burning(rng: &mut GlobalEntropy<WyRand>) -> Self {
        let duration = (rng.next_u32() % 30) as f32 * 0.1 + 1.0; // TODO: Needs to be shorter
        Self {
            kind: TreeActionKind::Burning,
            timer: Timer::from_seconds(duration, TimerMode::Repeating),
        }
    }

    pub fn felling(rng: &mut GlobalEntropy<WyRand>) -> Self {
        let duration = (rng.next_u32() % 30) as f32 * 0.1 + 1.0;
        Self {
            kind: TreeActionKind::Felling,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

impl TreeAction {
    fn trigger(&self, commands: &mut Commands, entity: Entity) {
        self.kind.trigger(commands, entity);
    }
}

fn handle_tree_action(
    mut commands: Commands,
    season: Res<Season>,
    time: Res<Time>,
    mut tree_action_q: Query<(Entity, &mut TreeAction)>,
) {
    if matches!(season.state, SeasonState::Simulation) {
        for (entity, mut tree_action) in &mut tree_action_q {
            if tree_action.timer.tick(time.delta()).just_finished() {
                tree_action.trigger(&mut commands, entity);

                // TODO: Could be nicer, but we only need to remove for grow
                if matches!(tree_action.kind, TreeActionKind::Growing) {
                    commands.entity(entity).remove::<TreeAction>();
                }
            }
        }
    }
}

#[derive(Debug, Event)]
pub struct Grow(Entity);

fn grow(trigger: Trigger<Grow>, mut tree_q: Query<&mut Tree>) {
    let entity = trigger.event().0;

    if let Ok(mut tree) = tree_q.get_mut(entity) {
        if let Some(next_tree) = tree.next() {
            *tree = next_tree;
        }
    }
}

#[derive(Debug, Event)]
pub struct Die(Entity);

fn die(
    trigger: Trigger<Die>,
    tile_pos_q: Query<&TilePos, With<Tree>>,
    mut despawn_tree_events: EventWriter<DespawnTree>,
) {
    let entity = trigger.event().0;

    if let Ok(tile_pos) = tile_pos_q.get(entity) {
        despawn_tree_events.send(DespawnTree {
            tile_pos: *tile_pos,
        });
    }
}

#[derive(Debug, Event)]
pub struct Burn(Entity);

//TODO: Split up somehow
fn burn(
    trigger: Trigger<Burn>,
    mut entity_map: Local<EntityHashMap<usize>>,
    tree_q: Query<(&Tree, &TilePos)>,
    tree_tile_storage_q: Query<&TileStorage, With<TreeLayer>>,
    mut commands: Commands,
    mut despawn_tree_events: EventWriter<DespawnTree>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    mut ground_q: Query<&mut Ground>,
    ground_tile_storage_q: Query<&TileStorage, With<GroundLayer>>,
) {
    let entity = trigger.event().0;

    if let Ok((tree, tile_pos)) = tree_q.get(entity) {
        let counter = entity_map.entry(entity).or_insert(0);
        *counter += 1;

        if *counter == 1 {
            // spread fire
            debug!("Spread fire from {:?}", tile_pos);
            let tile_storage = tree_tile_storage_q.single();
            Neighbors::get_square_neighboring_positions(tile_pos, &tile_storage.size, true)
                .entities(tile_storage)
                .iter()
                .for_each(|entity| {
                    commands
                        .entity(*entity)
                        .insert(TreeAction::burning(&mut rng)); // TODO: Only do this if they do not already have a burning component
                });
        } else {
            despawn_tree_events.send(DespawnTree {
                tile_pos: *tile_pos,
            });
            entity_map.remove(&entity);

            if matches!(tree, Tree::Mature | Tree::Overmature) {
                let tile_storage = ground_tile_storage_q.single();
                if let Some(entity) = tile_storage.get(tile_pos) {
                    if let Ok(mut ground) = ground_q.get_mut(entity) {
                        *ground = Ground::Nutrient;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Event)]
pub struct Fell(Entity);

fn fell(
    trigger: Trigger<Fell>,
    tree_q: Query<(&Tree, &TilePos)>,
    mut despawn_tree_events: EventWriter<DespawnTree>,
    mut score: ResMut<Score>,
    mut ground_q: Query<&mut Ground>,
    ground_tile_storage_q: Query<&TileStorage, With<GroundLayer>>,
) {
    let entity = trigger.event().0;

    if let Ok((tree, tile_pos)) = tree_q.get(entity) {
        despawn_tree_events.send(DespawnTree {
            tile_pos: *tile_pos,
        });

        let mut tree_score = tree.score();

        let tile_storage = ground_tile_storage_q.single();
        if let Some(entity) = tile_storage.get(tile_pos) {
            if let Ok(mut ground) = ground_q.get_mut(entity) {
                if matches!(*ground, Ground::Nutrient) {
                    tree_score *= 3;
                    *ground = Ground::Normal;
                }
            }
        }

        score.0 += tree_score;
    }
}

// Effects
// TODO: Not ideal this way
fn remove_effects(
    mut commands: Commands,
    mut effect_tile_storage_q: Query<&mut TileStorage, With<EffectLayer>>,
) {
    let mut tile_storage = effect_tile_storage_q.single_mut();

    for x in 0..tile_storage.size.x {
        for y in 0..tile_storage.size.y {
            let tile_pos = TilePos { x, y };
            if let Some(entity) = tile_storage.get(&tile_pos) {
                commands.entity(entity).despawn();
            }
            tile_storage.remove(&tile_pos);
        }
    }
}

fn handle_effects(
    tree_q: Query<(&TilePos, &TreeAction), Without<BadWeather>>,
    mut commands: Commands,
    mut effect_tile_storage_q: Query<(Entity, &mut TileStorage), With<EffectLayer>>,
) {
    let (tile_map_entity, mut tile_storage) = effect_tile_storage_q.single_mut();
    for (tile_pos, tree_action) in &tree_q {
        commands.entity(tile_map_entity).with_children(|parent| {
            let entity = parent
                .spawn(TileBundle {
                    position: *tile_pos,
                    texture_index: TileTextureIndex(tree_action.kind.effect_texture_index()),
                    tilemap_id: TilemapId(tile_map_entity),
                    ..default()
                })
                .id();

            tile_storage.set(tile_pos, entity);
        });
    }
}

fn handle_bad_weather(
    tree_q: Query<&TilePos, With<BadWeather>>,
    mut commands: Commands,
    mut effect_tile_storage_q: Query<(Entity, &mut TileStorage), With<EffectLayer>>,
) {
    let (tile_map_entity, mut tile_storage) = effect_tile_storage_q.single_mut();
    for tile_pos in &tree_q {
        commands.entity(tile_map_entity).with_children(|parent| {
            let entity = parent
                .spawn(TileBundle {
                    position: *tile_pos,
                    texture_index: TileTextureIndex(4),
                    tilemap_id: TilemapId(tile_map_entity),
                    ..default()
                })
                .id();

            tile_storage.set(tile_pos, entity);
        });
    }
}
