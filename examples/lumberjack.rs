use bevy::prelude::*;
use bevy_goap::{Action, ActionState, Actor, Condition, GoapPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GoapPlugin)
        .add_startup_system(create_lumberjack)
        .add_system(get_axe_action_system)
        .add_system(chop_tree_action_system)
        .add_system(collect_wood_action_system)
        .run();
}

fn create_lumberjack(mut commands: Commands) {
    // Considering only actions with conditions that are local to the actor (and not world) for the moment.

    let get_axe_action = Action::build(GetAxeAction)
        .with_precondition(ActorHasAxeCondition, false)
        .with_postcondition(ActorHasAxeCondition, true);

    let chop_tree_action = Action::build(ChopTreeAction::new(5))
        .with_precondition(ActorHasAxeCondition, true)
        .with_postcondition(ActorHasWoodCondition, true);

    let collect_wood_action = Action::build(CollectWoodAction)
        .with_precondition(ActorHasWoodCondition, false)
        .with_postcondition(ActorHasWoodCondition, true);

    // Try toggling the initial conditions and removing actions to observe the different action sequences the lumberjack performs!
    let lumberjack = Actor::build(Lumberjack)
        .with_initial_condition(ActorHasAxeCondition, false)
        .with_initial_condition(ActorHasWoodCondition, false)
        .with_goal(ActorHasWoodCondition, true)
        .with_action(get_axe_action)
        .with_action(collect_wood_action)
        .with_action(chop_tree_action);

    commands.spawn_empty().insert(lumberjack);
}

#[derive(Component, Clone)]
struct Lumberjack;

#[derive(Component, Clone)]
struct GetAxeAction;

fn get_axe_action_system(mut query: Query<&mut ActionState, With<GetAxeAction>>) {
    for mut action_state in query.iter_mut() {
        match *action_state {
            ActionState::Executing => {
                println!("Getting axe!");

                *action_state = ActionState::Complete;
            }
            ActionState::Complete => {
                println!("GetAxeAction is Complete.");
            }
            _ => {}
        };
    }
}

struct ActorHasAxeCondition;
impl Condition for ActorHasAxeCondition {}

#[derive(Component, Clone)]
struct ChopTreeAction {
    max_chops: u8,
    current_chops: u8,
}

impl ChopTreeAction {
    fn new(max_chops: u8) -> Self {
        Self {
            max_chops,
            current_chops: 0,
        }
    }
}

fn chop_tree_action_system(mut query: Query<(&mut ActionState, &mut ChopTreeAction)>) {
    for (mut action_state, mut chop_tree_action) in query.iter_mut() {
        if let ActionState::Executing = *action_state {
            chop_tree_action.current_chops += 1;
            println!("Chopped tree {} times!", chop_tree_action.current_chops);

            if chop_tree_action.current_chops >= chop_tree_action.max_chops {
                *action_state = ActionState::Complete;
            }
        };
    }
}

#[derive(Component, Clone)]
struct CollectWoodAction;

fn collect_wood_action_system(mut query: Query<&mut ActionState, With<CollectWoodAction>>) {
    for mut action_state in query.iter_mut() {
        if let ActionState::Executing = *action_state {
            println!("Collecting wood!");

            *action_state = ActionState::Complete;
        };
    }
}

struct ActorHasWoodCondition;
impl Condition for ActorHasWoodCondition {}
