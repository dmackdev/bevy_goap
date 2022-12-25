use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use bevy::prelude::{Commands, Component, Entity, EventReader, Query};
use pathfinding::prelude::astar;

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
) {
    let mut planning_state = planning_state_query.single_mut();

    for ev in ev_request_plan.iter() {
        println!("Received RequestPlanEvent");
        let mut should_queue = false;

        if let Ok(actor) = actors_query.get(ev.0) {
            for action_entity in actor.actions.iter() {
                let action_state_result = action_states_query.get_mut(*action_entity);

                if let Ok(mut action_state) = action_state_result {
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
    actions: Query<&Action>,
) {
    let mut new_queue: Vec<Entity> = vec![];

    for actor_entity in planning_state_query.single().queue.iter() {
        println!("Plan requested for {:?}", actor_entity);

        if let Ok(mut actor) = actors.get_mut(*actor_entity) {
            let all_actions_ready = actor.actions.iter().all(|action_entity| {
                matches!(
                    action_states.get(*action_entity),
                    Ok(ActionState::EvaluationSuccess) | Ok(ActionState::EvaluationFailure)
                )
            });

            if !all_actions_ready {
                // Not all the actions for this actor have finished evaluating, we must requeue the plan request for this actor to plan it later.
                new_queue.push(*actor_entity);
                continue;
            }

            let actor_action_nodes = actor
                .actions
                .iter()
                .filter_map(|action_entity| match action_states.get(*action_entity) {
                    // Only consider actions that have a succesful evaluation.
                    Ok(ActionState::EvaluationSuccess) => {
                        let action = actions.get(*action_entity).unwrap();

                        Some(Node {
                            id: NodeId::Action(*action_entity),
                            preconditions: action.preconditions.clone(),
                            postconditions: action.postconditions.clone(),
                        })
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            let start_node = Node {
                id: NodeId::Start,
                preconditions: GoapState::new(),
                postconditions: actor.current_state.clone(),
            };

            let goal_node = Node {
                id: NodeId::Goal,
                preconditions: actor.current_goal.clone(),
                postconditions: GoapState::new(),
            };

            let (node_path, _) = astar(
                &&start_node,
                |node| node.successors(&actor_action_nodes),
                |_| 1, // TODO: Need a heuristic. Alternatively, Dijkstra could be used to solely use the action's cost.
                |node| node.postconditions_match_preconditions_of(&goal_node), // This will exclude the goal node from the path.
            )
            .unwrap_or((vec![], 0));

            let action_path = node_path.iter().filter_map(|node| match node.id {
                NodeId::Action(e) => Some(e),
                _ => None,
            });

            actor.current_path = VecDeque::from_iter(action_path);

            if let Some(action_entity) = actor.current_path.front() {
                println!("Plan created for {:?}.", actor_entity);
                let mut action_state = action_states.get_mut(*action_entity).unwrap();
                *action_state = ActionState::Started;
            } else {
                println!("No plan available for {:?}.", actor_entity);

                for action_entity in actor.actions.iter() {
                    let mut action_state = action_states.get_mut(*action_entity).unwrap();
                    *action_state = ActionState::PlanFailure;
                }
            }
        }
    }

    planning_state_query.single_mut().queue = new_queue;
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
enum NodeId {
    Action(Entity),
    Start,
    Goal,
}

#[derive(Debug, Clone, Eq)]
struct Node {
    id: NodeId,
    preconditions: GoapState,
    postconditions: GoapState,
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Node {
    fn successors<'a>(&self, nodes: &'a [Node]) -> Vec<(&'a Node, i32)> {
        nodes
            .iter()
            .filter_map(|other| {
                if self.postconditions_match_preconditions_of(other) {
                    Some((other, 1)) // TODO: need to add action cost here.
                } else {
                    None
                }
            })
            .collect()
    }

    fn postconditions_match_preconditions_of(&self, other: &Node) -> bool {
        for (key, pre_cond) in other.preconditions.state.iter() {
            match self.postconditions.state.get(key) {
                Some(post_cond) if post_cond == pre_cond => continue,
                _ => return false,
            }
        }
        true
    }
}
