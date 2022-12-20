use bevy::prelude::*;
use bevy_goap::{Action, ActionState, Actor, Condition, GoapPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GoapPlugin)
        .add_startup_system(create_lumberjack)
        .add_system(find_axe_action_system)
        .add_system(move_to_tree_action_system)
        .run();
}

fn create_lumberjack(mut commands: Commands) {
    let find_axe_action =
        Action::build(FindAxeAction).with_precondition(IsAxeAvailableCondition, true);

    let move_to_tree_action =
        Action::build(MoveToTreeAction).with_precondition(IsTreeAvailableCondition, true);

    let lumberjack = Actor::build(Lumberjack)
        .with_action(find_axe_action)
        .with_action(move_to_tree_action);

    commands.spawn_empty().insert(lumberjack);
}

#[derive(Component, Clone)]
struct Lumberjack;

#[derive(Component, Clone)]
struct FindAxeAction;

fn find_axe_action_system(mut query: Query<&mut ActionState, With<FindAxeAction>>) {
    for mut action_state in query.iter_mut() {
        match *action_state {
            ActionState::Executing => {
                println!("Finding axe!");
                println!("Found axe!");

                *action_state = ActionState::Complete;
            }
            ActionState::Complete => {
                println!("FindAxeAction is Complete.");
            }
            _ => {}
        };
    }
}

struct IsAxeAvailableCondition;

impl Condition for IsAxeAvailableCondition {}

#[derive(Component, Clone)]
struct MoveToTreeAction;

fn move_to_tree_action_system(mut query: Query<&mut ActionState, With<MoveToTreeAction>>) {
    for mut action_state in query.iter_mut() {
        match *action_state {
            ActionState::Executing => {
                println!("Moving to tree!");
                println!("Is at tree!");

                *action_state = ActionState::Complete;
            }
            ActionState::Complete => {
                println!("MoveToTreeAction is Complete.");
            }
            _ => {}
        };
    }
}

struct IsTreeAvailableCondition;

impl Condition for IsTreeAvailableCondition {}
