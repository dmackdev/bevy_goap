use bevy::prelude::Entity;

use crate::{state::GoapState, Action};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlanNodeId {
    Start,
    Action(Entity),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanNode {
    pub id: PlanNodeId,
    current_state: GoapState,
}

impl PlanNode {
    pub fn get_initial(initial_state: &GoapState) -> PlanNode {
        PlanNode {
            id: PlanNodeId::Start,
            current_state: initial_state.clone(),
        }
    }

    fn get_next(prev_state: &GoapState, action: &Action, action_entity: Entity) -> PlanNode {
        let mut next_state = prev_state.clone();
        next_state.extend(action.postconditions.clone());

        PlanNode {
            id: PlanNodeId::Action(action_entity),
            current_state: next_state,
        }
    }

    pub fn get_successors(&self, actions: &[(&Action, &Entity)]) -> Vec<(PlanNode, i32)> {
        actions
            .iter()
            .filter_map(|(action, action_entity)| {
                self.matches(&action.preconditions).then_some((
                    PlanNode::get_next(&self.current_state, action, **action_entity),
                    action.cost,
                ))
            })
            .collect()
    }

    pub fn mismatch_count(&self, target: &GoapState) -> i32 {
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

    pub fn matches(&self, target: &GoapState) -> bool {
        self.mismatch_count(target) == 0
    }
}
