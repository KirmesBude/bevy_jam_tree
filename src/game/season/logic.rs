use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

use crate::{
    game::{
        spawn::{
            level::TreeLayer,
            tree::{grow_logic, overcrowd_dying_logic, DespawnTree, Tree},
        },
        Score,
    },
    screen::Screen,
};

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
        (handle_tree_action, spread_fire).run_if(in_state(Screen::Playing)),
    );
}

// Spring, Summer, Autumn
#[derive(Debug, Event)]
pub struct SetupGrowing;

fn setup_growing(
    _trigger: Trigger<SetupGrowing>,
    mut commands: Commands,
    tree_tile_storage_q: Query<&TileStorage, With<TreeLayer>>,
    tree_q: Query<(Entity, &Tree, &TilePos)>,
) {
    grow_logic(&mut commands, tree_tile_storage_q.single(), tree_q);
}

// Spring, Autumn
#[derive(Debug, Event)]
pub struct SetupOvercrowdDying;

fn setup_overcrowd_dying(
    _trigger: Trigger<SetupOvercrowdDying>,
    mut commands: Commands,
    tree_tile_storage_q: Query<&TileStorage, With<TreeLayer>>,
    tree_q: Query<(Entity, &Tree, &TilePos)>,
) {
    overcrowd_dying_logic(&mut commands, tree_tile_storage_q.single(), tree_q);
}

// Winter
#[derive(Debug, Event)]
pub struct SetupSeedlingDying;

fn setup_seedling_dying(
    _trigger: Trigger<SetupSeedlingDying>,
    mut commands: Commands,
    tree_q: Query<(Entity, &Tree)>,
) {
    for (entity, tree) in &tree_q {
        if matches!(tree, Tree::Seedling) {
            commands.entity(entity).insert(TreeAction::dying());
        }
    }
}

#[derive(Debug, Event)]
pub struct SetupFelling;

fn setup_felling(
    _trigger: Trigger<SetupFelling>,
    mut commands: Commands,
    tree_q: Query<(Entity, &Tree)>,
) {
    for (entity, tree) in &tree_q {
        if matches!(tree, Tree::Mature | Tree::Overmature) {
            commands.entity(entity).insert(TreeAction::felling());
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
}

#[derive(Debug, Component, Reflect)]
pub struct TreeAction {
    kind: TreeActionKind,
    timer: Timer,
}

impl TreeAction {
    pub fn growing() -> Self {
        Self {
            kind: TreeActionKind::Growing,
            timer: Timer::from_seconds(3.0, TimerMode::Once), /* TODO: Random */
        }
    }

    pub fn dying() -> Self {
        Self {
            kind: TreeActionKind::Dying,
            timer: Timer::from_seconds(3.0, TimerMode::Once), /* TODO: Random */
        }
    }

    pub fn _burning() -> Self {
        Self {
            kind: TreeActionKind::Burning,
            timer: Timer::from_seconds(3.0, TimerMode::Once), /* TODO: Random */
        }
    }

    pub fn felling() -> Self {
        Self {
            kind: TreeActionKind::Felling,
            timer: Timer::from_seconds(3.0, TimerMode::Once), /* TODO: Random */
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
    time: Res<Time>,
    mut tree_action_q: Query<(Entity, &mut TreeAction)>,
) {
    for (entity, mut tree_action) in &mut tree_action_q {
        if tree_action.timer.tick(time.delta()).just_finished() {
            tree_action.trigger(&mut commands, entity);

            commands.entity(entity).remove::<TreeAction>();
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

fn burn(trigger: Trigger<Burn>) {
    let _entity = trigger.event().0;
}

#[derive(Debug, Event)]
pub struct Fell(Entity);

fn fell(
    trigger: Trigger<Fell>,
    tree_q: Query<(&Tree, &TilePos)>,
    mut despawn_tree_events: EventWriter<DespawnTree>,
    mut score: ResMut<Score>,
) {
    let entity = trigger.event().0;

    if let Ok((tree, tile_pos)) = tree_q.get(entity) {
        despawn_tree_events.send(DespawnTree {
            tile_pos: *tile_pos,
        });

        score.0 += tree.score();
    }
}

fn spread_fire() {}
