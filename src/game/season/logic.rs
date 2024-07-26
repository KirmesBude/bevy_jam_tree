use bevy::prelude::*;

use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<TreeAction>();

    app.observe(setup_growing);
    app.observe(setup_overcrowd_dying);
    app.observe(setup_multiply);
    app.observe(setup_seedling_dying);
    app.observe(setup_felling);
    
    app.observe(grow);
    app.observe(die);
    app.observe(burn);
    app.observe(multiply);
    app.observe(fell);

    app.add_systems(Update, (handle_tree_action, spread_fire).run_if(in_state(Screen::Playing)));
}

// Spring, Summer, Autumn
#[derive(Debug, Event)]
pub struct SetupGrowing;

fn setup_growing(_trigger: Trigger<SetupGrowing>) {

}

// Spring, Autumn
#[derive(Debug, Event)]
pub struct SetupOvercrowdDying;

fn setup_overcrowd_dying(_trigger: Trigger<SetupOvercrowdDying>) {
    
}

// Autumn
#[derive(Debug, Event)]
pub struct SetupMultiply;

fn setup_multiply(_trigger: Trigger<SetupMultiply>) {
    // Spawn trees where relevant with growing component
}

// Winter
#[derive(Debug, Event)]
pub struct SetupSeedlingDying;

fn setup_seedling_dying(_trigger: Trigger<SetupSeedlingDying>) {
    
}

#[derive(Debug, Event)]
pub struct SetupFelling;

fn setup_felling(_trigger: Trigger<SetupFelling>) {
    
}

#[derive(Debug, Clone, Copy, Reflect)]
enum TreeActionKind {
    Growing,
    Dying,
    Burning,
    Multiplying,
    Felling,
}

impl TreeActionKind {
    fn trigger(&self, commands: &mut Commands, entity: Entity) {
        match self {
            TreeActionKind::Growing => commands.trigger(Grow(entity)),
            TreeActionKind::Dying => commands.trigger(Die(entity)),
            TreeActionKind::Burning => commands.trigger(Burn(entity)),
            TreeActionKind::Multiplying => commands.trigger(Multiply(entity)),
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
    fn trigger(&self, commands: &mut Commands, entity: Entity) {
        self.kind.trigger(commands, entity);
    }
}

fn handle_tree_action(mut commands: Commands, time: Res<Time>, mut tree_action_q: Query<(Entity, &mut TreeAction)>) {
    for (entity, mut tree_action) in &mut tree_action_q {
        if tree_action.timer.tick(time.delta()).just_finished() {
            tree_action.trigger(&mut commands, entity);

            commands.entity(entity).remove::<TreeAction>();
        }
    }
}

#[derive(Debug, Event)]
pub struct Grow(Entity);

fn grow(_trigger: Trigger<Grow>) {

}

#[derive(Debug, Event)]
pub struct Die(Entity);

fn die(_trigger: Trigger<Die>) {
    
}

#[derive(Debug, Event)]
pub struct Burn(Entity);

fn burn(_trigger: Trigger<Burn>) {
    
}

#[derive(Debug, Event)]
pub struct Multiply(Entity);

fn multiply(_trigger: Trigger<Multiply>) {
    
}

#[derive(Debug, Event)]
pub struct Fell(Entity);

fn fell(_trigger: Trigger<Fell>) {
    
}

fn spread_fire() {

}