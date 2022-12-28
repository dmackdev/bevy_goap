use std::{any::TypeId, collections::HashMap};

use bevy::prelude::{App, Component, Entity, Query, SystemSet, With};
use bevy_goap::{
    Action, ActionState, Actor, ActorState, Condition, EvaluationResult, GoapPlugin, GoapStage,
};

#[derive(Component, Clone)]
struct Lumberjack;

#[derive(Component, Clone)]
struct GetAxeAction;

#[derive(Component, Clone)]
struct ChopTreeAction;

#[derive(Component, Clone)]
struct CollectWoodAction;

struct HasAxeCondition;
impl Condition for HasAxeCondition {}

struct HasWoodCondition;
impl Condition for HasWoodCondition {}

fn create_lumberjack(app: &mut App) -> Entity {
    let get_axe_action = Action::build(GetAxeAction)
        .with_precondition(HasAxeCondition, false)
        .with_postcondition(HasAxeCondition, true);

    let chop_tree_action = Action::build(ChopTreeAction)
        .with_precondition(HasAxeCondition, true)
        .with_postcondition(HasWoodCondition, true);

    let collect_wood_action =
        Action::build(CollectWoodAction).with_postcondition(HasWoodCondition, true);

    let lumberjack = Actor::build(Lumberjack)
        .with_initial_condition(HasAxeCondition, false)
        .with_initial_condition(HasWoodCondition, false)
        .with_goal(HasWoodCondition, true)
        .with_action(get_axe_action)
        .with_action(chop_tree_action)
        .with_action(collect_wood_action);

    app.world.spawn(lumberjack).id()
}

#[test]
fn integration() {
    let mut app = App::new();
    app.add_plugin(GoapPlugin);
    app.add_system_set_to_stage(
        GoapStage::Actions,
        SystemSet::new()
            .with_system(action_system::<GetAxeAction>)
            .with_system(action_system::<ChopTreeAction>)
            .with_system(action_system::<CollectWoodAction>),
    );

    let mut actor_test_case = ActorTestCase::new(ActorState::CompletedPlan);

    actor_test_case.insert_action_test_case::<GetAxeAction>(ActionTestCase {
        new_cost: 1,
        evaluation_result: EvaluationResult::Success,
        execution_result: ActionState::Complete,
    });

    actor_test_case.insert_action_test_case::<ChopTreeAction>(ActionTestCase {
        new_cost: 1,
        evaluation_result: EvaluationResult::Success,
        execution_result: ActionState::Complete,
    });

    // For this action test case we set a higher cost than the two above actions combined (which together achieve the goal), so we do not expect it to be in the path.
    actor_test_case.insert_action_test_case::<CollectWoodAction>(ActionTestCase {
        new_cost: 3,
        evaluation_result: EvaluationResult::Success,
        execution_result: ActionState::Complete, // Uneeded
    });

    actor_test_case.expect_next_action_in_path_to_be::<GetAxeAction>();
    actor_test_case.expect_next_action_in_path_to_be::<ChopTreeAction>();

    app.world.spawn(actor_test_case.clone());

    // No longer need mutatations.
    let actor_test_case = actor_test_case;

    create_lumberjack(&mut app);

    // Build the actor and its actions.
    app.update();

    assert_eq!(app.world.query::<&Actor>().iter(&app.world).len(), 1);
    assert_eq!(
        app.world.query::<&Action>().iter(&app.world).len(),
        actor_test_case.action_test_cases.len()
    );

    // Actor should be waiting for a plan.
    assert_eq!(
        app.world
            .query::<(&ActorState, With<Lumberjack>)>()
            .single(&app.world)
            .0,
        &ActorState::AwaitingPlan
    );

    // ActionStates should be Evaulate.
    assert!(app
        .world
        .query::<&ActionState>()
        .iter(&app.world)
        .all(|action_state| *action_state == ActionState::Evaluate));

    // Let the Actions finish evaluation, the plan to be created, and the first action to be started.
    app.update();

    // Actor should now be executing a plan.
    assert_eq!(
        app.world
            .query::<(&ActorState, With<Lumberjack>)>()
            .single(&app.world)
            .0,
        &ActorState::ExecutingPlan
    );

    for _ in 0..actor_test_case.expected_path.len() {
        // Let the current Action finish the Start action state.
        app.update();

        // Let the current Action finish the Executing action state.
        app.update();
    }

    // Actor should now have finished their plan.
    assert_eq!(
        app.world
            .query::<(&ActorState, With<Lumberjack>)>()
            .single(&app.world)
            .0,
        &actor_test_case.expected_final_actor_state
    );

    // All Actions should now be Idle.
    assert!(app
        .world
        .query::<&ActionState>()
        .iter(&app.world)
        .all(|action_state| *action_state == ActionState::Idle));
}

fn action_system<T: Component>(
    mut action_query: Query<(&mut Action, &mut ActionState), With<T>>,
    mut actor_test_case_query: Query<&mut ActorTestCase>,
) {
    let action_test_case = actor_test_case_query
        .single()
        .get_test_case_for_action::<T>();
    let mut actor_test_case = actor_test_case_query.single_mut();

    for (mut action, mut action_state) in action_query.iter_mut() {
        match *action_state {
            ActionState::Evaluate => {
                println!("Action evaluating.");

                action.update_cost(action_test_case.new_cost);
                *action_state = ActionState::EvaluationComplete(action_test_case.evaluation_result);
            }
            ActionState::NotInPlan(_) => {
                println!("Action not in plan.");

                assert_not_in_expected_path::<T>(&mut actor_test_case);

                *action_state = ActionState::Idle;
            }
            ActionState::Started => {
                println!("Action starting.");

                assert_correct_idx_in_path::<T>(&mut actor_test_case);

                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                println!("Action executing.");

                assert_correct_idx_in_path::<T>(&mut actor_test_case);

                finish_action(
                    actor_test_case.as_mut(),
                    &action_test_case,
                    action_state.as_mut(),
                );
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
struct ActionTestCase {
    new_cost: u32,
    evaluation_result: EvaluationResult,
    execution_result: ActionState,
}

#[derive(Component, Clone)]
struct ActorTestCase {
    action_test_cases: HashMap<TypeId, ActionTestCase>,
    current_action_idx: usize,
    expected_path: Vec<TypeId>,
    expected_final_actor_state: ActorState,
}

impl ActorTestCase {
    fn new(expected_final_actor_state: ActorState) -> Self {
        Self {
            action_test_cases: HashMap::new(),
            current_action_idx: 0,
            expected_path: Vec::new(),
            expected_final_actor_state,
        }
    }

    fn expect_next_action_in_path_to_be<T: Component>(&mut self) {
        self.expected_path.push(TypeId::of::<T>());
    }

    fn insert_action_test_case<T: Component>(&mut self, action_test_cases: ActionTestCase) {
        self.action_test_cases
            .insert(TypeId::of::<T>(), action_test_cases);
    }

    fn get_test_case_for_action<T: Component>(&self) -> ActionTestCase {
        self.action_test_cases
            .get(&TypeId::of::<T>())
            .unwrap()
            .clone()
    }
}

fn assert_correct_idx_in_path<T: Component>(actor_test_case: &mut ActorTestCase) {
    assert_eq!(
        *actor_test_case
            .expected_path
            .get(actor_test_case.current_action_idx)
            .expect("An Action was completed, but it was not expected to have executed!"),
        TypeId::of::<T>()
    );
}

fn assert_not_in_expected_path<T: Component>(actor_test_case: &mut ActorTestCase) {
    assert!(!actor_test_case.expected_path.contains(&TypeId::of::<T>()));
}

fn finish_action(
    actor_test_case: &mut ActorTestCase,
    test_case: &ActionTestCase,
    action_state: &mut ActionState,
) {
    actor_test_case.current_action_idx += 1;
    *action_state = test_case.execution_result;
}
