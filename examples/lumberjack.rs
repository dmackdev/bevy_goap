use bevy::prelude::*;
use bevy_goap::{ActionState, Actor, GoapPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(GoapPlugin)
        .add_startup_system(create_lumberjack)
        .add_system(find_axe_action_system)
        .add_system(move_to_tree_action_system)
        .run();
}

fn create_lumberjack(mut commands: Commands) {
    commands.spawn_empty().insert(
        Actor::build(Lumberjack)
            .with_action(FindAxeAction)
            .with_action(MoveToTreeAction),
    );
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
