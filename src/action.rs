use std::{any::TypeId, collections::HashMap, sync::Arc};

use bevy::prelude::{Commands, Component, Entity, Query};

use crate::{actor::Actor, common::MarkerComponent, condition::Condition, WorldCondition};

#[derive(Component)]
pub enum ActionState {
    Idle,
    Executing,
    Complete,
}

#[derive(Component, Clone)]
pub struct Action {
    actor_entity: Entity,
    pub(crate) preconditions: HashMap<TypeId, bool>,
    pub(crate) world_preconditions: HashMap<TypeId, bool>,
    pub(crate) postconditions: HashMap<TypeId, bool>,
}

impl Action {
    pub fn build(marker_component: impl MarkerComponent + 'static) -> ActionBuilder {
        ActionBuilder {
            marker_component: Arc::new(marker_component),
            preconditions: HashMap::new(),
            world_preconditions: HashMap::new(),
            postconditions: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct ActionBuilder {
    marker_component: Arc<dyn MarkerComponent>,
    preconditions: HashMap<TypeId, bool>,
    world_preconditions: HashMap<TypeId, bool>,
    postconditions: HashMap<TypeId, bool>,
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

    pub fn with_world_precondition<T: WorldCondition + 'static>(
        mut self,
        value: bool,
    ) -> ActionBuilder {
        self.world_preconditions.insert(TypeId::of::<T>(), value);
        self
    }

    pub fn with_postcondition<T: Condition + 'static>(
        mut self,
        _postcondition: T,
        value: bool,
    ) -> ActionBuilder {
        self.postconditions.insert(TypeId::of::<T>(), value);
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
                world_preconditions: self.world_preconditions.clone(),
                preconditions: self.preconditions.clone(),
                postconditions: self.postconditions.clone(),
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
    let mut completed = vec![];

    for (action, mut action_state) in query.iter_mut() {
        if let ActionState::Complete = *action_state {
            *action_state = ActionState::Idle;

            completed.push((action.actor_entity, action.postconditions.clone()));
        };
    }

    for (actor_entity, postconditions) in completed {
        let mut actor = actors.get_mut(actor_entity).unwrap();

        if let Some(next_action_entity) = actor.complete_action(postconditions) {
            let (_, mut next_action_state) = query.get_mut(*next_action_entity).unwrap();
            *next_action_state = ActionState::Executing;
        }
    }
}
