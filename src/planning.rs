use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use bevy::prelude::{Entity, EventReader, Query};
use pathfinding::prelude::astar;

use crate::state::{GoapState, GoapWorldState};
use crate::{
    action::{Action, ActionState},
    actor::Actor,
};

pub struct RequestPlanEvent(pub(crate) Entity);

// TODO: Introduce a planning "queue" which queues up the plan request events.
// The next plan request can be handled when the action started by the previous plan transitions out of ActionState::Started.
// Probably need to introduce a ActionState::StartFinished to detect moving out of ActionState::Started.
// This is because an Action's start phase may modify the world conditions, which could affect the next plan.
pub fn request_plan_event_handler_system(
    mut ev_request_plan: EventReader<RequestPlanEvent>,
    mut actors: Query<&mut Actor>,
    mut action_states: Query<&mut ActionState>,
    actions: Query<&Action>,
    world_state_query: Query<&GoapWorldState>,
) {
    let world_state = world_state_query.single();

    for ev in ev_request_plan.iter() {
        println!("Plan requested for {:?}", ev.0);

        if let Ok(mut actor) = actors.get_mut(ev.0) {
            println!("Updating path for actor");

            let actor_action_nodes = actor
                .actions
                .iter()
                .enumerate()
                .map(|(idx, action_entity)| {
                    let action = actions.get(*action_entity).unwrap();

                    let mut preconditions = action.preconditions.clone();
                    preconditions.extend(action.world_preconditions.clone());

                    Node {
                        id: idx + 1,
                        action_entity: Some(*action_entity),
                        preconditions,
                        postconditions: action.postconditions.clone(),
                    }
                })
                .collect::<Vec<_>>();

            let mut start_postconditions = actor.current_state.clone();
            start_postconditions.extend(world_state.get());

            let start_node = Node {
                id: 0,
                action_entity: None,
                preconditions: GoapState::new(),
                postconditions: start_postconditions,
            };

            let goal_node = Node {
                id: actor_action_nodes.len() + 1,
                action_entity: None,
                preconditions: actor.current_goal.clone(),
                postconditions: GoapState::new(),
            };

            let (node_path, _) = astar(
                &start_node,
                |node| node.successors(&actor_action_nodes),
                |_| 1, // TODO: Need a heuristic. Alternatively, Dijkstra could be used to solely use the action's cost.
                |node| node.postconditions_match_preconditions_of(&goal_node), // This will exclude the goal node from the path.
            )
            .unwrap_or((vec![], 0));

            // start_node does not have a populated action_entity, so it will be filtered out from the path here.
            let action_path = node_path.iter().filter_map(|node| node.action_entity);

            actor.current_path = VecDeque::from_iter(action_path);

            if let Some(action_entity) = actor.current_path.front() {
                let mut action_state = action_states.get_mut(*action_entity).unwrap();
                *action_state = ActionState::Started;
            }
        }
    }
}

#[derive(Debug, Clone, Eq)]
struct Node {
    id: usize,
    action_entity: Option<Entity>,
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
    fn successors(&self, nodes: &[Node]) -> Vec<(Node, i32)> {
        nodes
            .iter()
            .filter_map(|other| {
                if self.postconditions_match_preconditions_of(other) {
                    Some((other.clone(), 1)) // TODO: need to add action cost here.
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
