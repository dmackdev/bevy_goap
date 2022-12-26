use std::collections::VecDeque;
use std::hash::Hash;

use bevy::prelude::{Commands, Component, Entity, EventReader, Query};
use pathfinding::prelude::astar;

use crate::action::EvaluationResult;
use crate::actor::ActorState;
use crate::state::GoapState;
use crate::{
    action::{Action, ActionState},
    actor::Actor,
};

pub struct RequestPlanEvent(pub(crate) Entity);

#[derive(Component, Default, Debug)]
pub struct PlanningState {
    queue: Vec<Entity>,
}

pub fn create_planning_state(mut commands: Commands) {
    commands.spawn_empty().insert(PlanningState::default());
}

pub fn request_plan_event_handler_system(
    mut ev_request_plan: EventReader<RequestPlanEvent>,
    mut planning_state_query: Query<&mut PlanningState>,
    actors_query: Query<&Actor>,
    mut action_states_query: Query<&mut ActionState>,
    actions_query: Query<&Action>,
) {
    let mut planning_state = planning_state_query.single_mut();

    for ev in ev_request_plan.iter() {
        println!("Received RequestPlanEvent");
        let mut should_queue = false;

        if let Ok(actor) = actors_query.get(ev.0) {
            for action_entity in actor.actions.iter() {
                let action = actions_query.get(*action_entity).unwrap();
                let mut action_state = action_states_query.get_mut(*action_entity).unwrap();

                // If the action's postconditions already satisfy the actor's current state, we do not need to evaluate the action or consider it for the plan.
                let mut action_postconditions_already_satisfied = true;
                for (postcondition_key, postcondition_value) in action.postconditions.state.iter() {
                    match actor.current_state.state.get(postcondition_key) {
                        Some(current_state_value) if current_state_value != postcondition_value => {
                            action_postconditions_already_satisfied = false;
                            break;
                        }
                        None => {
                            action_postconditions_already_satisfied = false;
                            break;
                        }
                        _ => {}
                    };
                }

                if action_postconditions_already_satisfied {
                    *action_state = ActionState::EvaluationComplete(EvaluationResult::Skipped);
                } else {
                    // Since we have found at least one action that can be in the plan, we can queue this request.
                    should_queue = true;
                    *action_state = ActionState::Evaluate;
                }
            }
        }

        if should_queue {
            println!("Pushing {:?} to queue", ev.0);
            planning_state.queue.push(ev.0);
        }
    }
}

pub fn create_plan_system(
    mut planning_state_query: Query<&mut PlanningState>,
    mut actors: Query<&mut Actor>,
    mut action_states: Query<&mut ActionState>,
    mut actor_states: Query<&mut ActorState>,
    actions: Query<&Action>,
) {
    let mut new_queue: Vec<Entity> = vec![];

    for actor_entity in planning_state_query.single().queue.iter() {
        println!("Plan requested for {:?}", actor_entity);

        if let Ok(mut actor) = actors.get_mut(*actor_entity) {
            let all_actions_ready = actor.actions.iter().all(|action_entity| {
                matches!(
                    action_states.get(*action_entity),
                    Ok(ActionState::EvaluationComplete(_))
                )
            });

            if !all_actions_ready {
                println!(
                    "Not all actions are ready for {:?}, re-queueing request",
                    actor_entity
                );
                // Not all the actions for this actor have finished evaluating, we must requeue the plan request for this actor to plan it later.
                new_queue.push(*actor_entity);
                continue;
            }

            let actor_actions = actor
                .actions
                .iter()
                .filter_map(|action_entity| match action_states.get(*action_entity) {
                    // Only consider actions that have a succesful evaluation.
                    Ok(ActionState::EvaluationComplete(EvaluationResult::Success)) => {
                        let action = actions.get(*action_entity).unwrap();

                        Some((action, action_entity))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            let start_node = Node::get_initial(&actor.current_state);

            let (node_path, _) = astar(
                &start_node,
                |node| node.get_successors(&actor_actions),
                |node| node.mismatch_count(&actor.current_goal),
                |node| node.matches(&actor.current_goal),
            )
            .unwrap_or((vec![], 0));

            let action_path = node_path.iter().filter_map(|node| match node.id {
                NodeId::Action(e) => Some(e),
                _ => None,
            });

            actor.current_path = VecDeque::from_iter(action_path);

            if let Some(first_action_entity) = actor.current_path.front() {
                println!("Plan created for {:?}.", actor_entity);

                for action_entity in actor.actions.iter() {
                    let mut action_state = action_states.get_mut(*action_entity).unwrap();

                    if action_entity == first_action_entity {
                        *action_state = ActionState::Started;
                    } else if actor.current_path.contains(action_entity) {
                        *action_state = ActionState::WaitingToStart;
                    } else {
                        action_state.mark_not_in_plan();
                    }
                }

                let mut actor_state = actor_states.get_mut(*actor_entity).unwrap();
                *actor_state = ActorState::ExecutingPlan;
            } else {
                println!("No plan available for {:?}.", actor_entity);

                for action_entity in actor.actions.iter() {
                    let mut action_state = action_states.get_mut(*action_entity).unwrap();
                    action_state.mark_not_in_plan();
                }

                let mut actor_state = actor_states.get_mut(*actor_entity).unwrap();
                *actor_state = ActorState::NoPlanAvailable;
            }
        }
    }

    planning_state_query.single_mut().queue = new_queue;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NodeId {
    Start,
    Action(Entity),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node {
    id: NodeId,
    current_state: GoapState,
}

impl Node {
    fn get_initial(initial_state: &GoapState) -> Node {
        Node {
            id: NodeId::Start,
            current_state: initial_state.clone(),
        }
    }

    fn get_next(prev_state: &GoapState, action: &Action, action_entity: Entity) -> Node {
        let mut next_state = prev_state.clone();
        next_state.extend(action.postconditions.clone());

        Node {
            id: NodeId::Action(action_entity),
            current_state: next_state,
        }
    }

    fn get_successors(&self, actions: &[(&Action, &Entity)]) -> Vec<(Node, i32)> {
        actions
            .iter()
            .filter_map(|(action, action_entity)| {
                self.matches(&action.preconditions).then_some((
                    Node::get_next(&self.current_state, action, **action_entity),
                    action.cost,
                ))
            })
            .collect()
    }

    fn mismatch_count(&self, target: &GoapState) -> i32 {
        let mut count = 0;

        for (key, target_value) in target.state.iter() {
            match self.current_state.state.get(key) {
                Some(current_value) if current_value != target_value => {
                    count += 1;
                }
                None => {
                    count += 1;
                }
                _ => {}
            }
        }

        count
    }

    fn matches(&self, target: &GoapState) -> bool {
        self.mismatch_count(target) == 0
    }
}
