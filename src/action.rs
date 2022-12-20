use std::{any::TypeId, collections::HashMap, sync::Arc};

use bevy::prelude::{Commands, Component, Entity, Query};

use crate::{actor::Actor, common::MarkerComponent, condition::Condition};

#[derive(Component)]
pub enum ActionState {
    Idle,
    Executing,
    Complete,
}

#[derive(Component)]
pub struct Action {
    actor_entity: Entity,
    pub(crate) preconditions: HashMap<TypeId, bool>,
}

impl Action {
    pub fn build(marker_component: impl MarkerComponent + 'static) -> ActionBuilder {
        ActionBuilder {
            marker_component: Arc::new(marker_component),
            preconditions: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct ActionBuilder {
    marker_component: Arc<dyn MarkerComponent>,
    preconditions: HashMap<TypeId, bool>,
}

impl ActionBuilder {
    pub fn with_precondition<T: Condition + 'static>(
        mut self,
        _precondition: T,
        value: bool,
    ) -> ActionBuilder {
        self.preconditions.insert(TypeId::of::<T>(), value);
        self
    }
}

pub trait BuildAction: Send + Sync {
    fn build(&self, commands: &mut Commands, actor_entity: Entity) -> Entity;
}

impl BuildAction for ActionBuilder {
    fn build(&self, commands: &mut Commands, actor_entity: Entity) -> Entity {
        let action_entity = commands
            .spawn_empty()
            .insert(Action {
                actor_entity,
                preconditions: self.preconditions.clone(),
            })
            .insert(ActionState::Idle)
            .id();

        self.marker_component.insert(commands, action_entity);

        action_entity
    }
}

pub fn handle_completed_actions_system(
    mut actors: Query<&mut Actor>,
    mut query: Query<(&Action, &mut ActionState)>,
) {
    let mut actor_entities_to_start_next_action = vec![];

    for (action, mut action_state) in query.iter_mut() {
        if let ActionState::Complete = *action_state {
            *action_state = ActionState::Idle;

            actor_entities_to_start_next_action.push(action.actor_entity);
        };
    }

    for actor_entity in actor_entities_to_start_next_action {
        let mut actor = actors.get_mut(actor_entity).unwrap();

        actor.current_path.pop_front();

        if let Some(action_entity) = actor.current_path.front() {
            let (_, mut action_state) = query.get_mut(*action_entity).unwrap();
            *action_state = ActionState::Executing;
        }
    }
}
