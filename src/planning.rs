use std::hash::{Hash, Hasher};
use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
};

use bevy::prelude::{Entity, EventReader, Query};
use pathfinding::prelude::astar;

use crate::{
    action::{Action, ActionState},
    actor::Actor,
};

pub struct RequestPlanEvent(pub(crate) Entity);

pub fn request_plan_event_handler_system(
    mut ev_levelup: EventReader<RequestPlanEvent>,
    mut actors: Query<&mut Actor>,
    mut action_states: Query<&mut ActionState>,
    actions: Query<&Action>,
) {
    for ev in ev_levelup.iter() {
        println!("Plan requested for {:?}", ev.0);

        if let Ok(mut actor) = actors.get_mut(ev.0) {
            println!("Updating path for actor");

            let actor_action_nodes = actor
                .actions
                .iter()
                .enumerate()
                .map(|(idx, action_entity)| {
                    let action = actions.get(*action_entity).unwrap();
                    Node {
                        id: idx + 1,
                        action_entity: Some(*action_entity),
                        preconditions: action.preconditions.clone(),
                        postconditions: action.postconditions.clone(),
                    }
                })
                .collect::<Vec<_>>();

            let start_node = Node {
                id: 0,
                action_entity: None,
                preconditions: HashMap::new(),
                postconditions: actor.current_state.clone(),
            };

            let goal_node = Node {
                id: actor_action_nodes.len() + 1,
                action_entity: None,
                preconditions: actor.current_goal.clone(),
                postconditions: HashMap::new(),
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
                *action_state = ActionState::Executing;
            }
        }
    }
}

#[derive(Debug, Clone, Eq)]
struct Node {
    id: usize,
    action_entity: Option<Entity>,
    preconditions: HashMap<TypeId, bool>,
    postconditions: HashMap<TypeId, bool>,
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
        for (key, pre_cond) in other.preconditions.iter() {
            match self.postconditions.get(key) {
                Some(post_cond) if post_cond == pre_cond => continue,
                _ => return false,
            }
        }
        true
    }
}
