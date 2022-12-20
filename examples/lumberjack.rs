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
    // Considering actions local to actor for the moment

    // Preconditions: actor does not have axe
    // Postconditions: actor has axe
    let get_axe_action = Action::build(GetAxeAction).with_precondition(ActorHasAxeCondition, false);

    // Preconditions: actor has axe
    // Postconditions: actor has wood
    let chop_tree_action =
        Action::build(ChopTreeAction).with_precondition(ActorHasAxeCondition, true);

    // Preconditions: actor does not have wood
    // Postconditions: actor has wood
    let collect_wood_action =
        Action::build(CollectWoodAction).with_precondition(ActorHasWoodCondition, false);

    // Possible paths:
    // 1: GetAxeAction -> ChopTreeAction
    // 2: GetAxeAction -> CollectWoodAction (next lowest cost if axe and wood are both already available)
    // 3: CollectWoodAction (lowest cost if wood is already available)

    // Goal: actor has wood
    let lumberjack = Actor::build(Lumberjack)
        .with_action(get_axe_action)
        .with_action(chop_tree_action)
        .with_action(collect_wood_action);

    commands.spawn_empty().insert(lumberjack);
}

#[derive(Component, Clone)]
struct Lumberjack;

#[derive(Component, Clone)]
struct GetAxeAction;

fn find_axe_action_system(mut query: Query<&mut ActionState, With<GetAxeAction>>) {
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

struct ActorHasAxeCondition;
impl Condition for ActorHasAxeCondition {}

#[derive(Component, Clone)]
struct ChopTreeAction;

fn move_to_tree_action_system(mut query: Query<&mut ActionState, With<ChopTreeAction>>) {
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

#[derive(Component, Clone)]
struct CollectWoodAction;

struct ActorHasWoodCondition;
impl Condition for ActorHasWoodCondition {}
