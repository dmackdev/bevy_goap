use std::{collections::VecDeque, sync::Arc};

use bevy::prelude::{Added, Commands, Component, Entity, EventWriter, Query};

use crate::{action::BuildAction, common::MarkerComponent, RequestPlanEvent};

#[derive(Component)]
pub struct Actor {
    pub(crate) actions: Vec<Entity>,
    pub(crate) current_path: VecDeque<Entity>,
}

impl Actor {
    pub fn build(marker_component: impl MarkerComponent + 'static) -> ActorBuilder {
        ActorBuilder {
            marker_component: Arc::new(marker_component),
            actions: vec![],
        }
    }
}

#[derive(Component)]
pub struct ActorBuilder {
    marker_component: Arc<dyn MarkerComponent>,
    actions: Vec<Arc<dyn BuildAction>>,
}
impl ActorBuilder {
    pub fn with_action(mut self, action: impl BuildAction + 'static) -> Self {
        self.actions.push(Arc::new(action));
        self
    }

    fn build(&self, commands: &mut Commands, actor_entity: Entity) {
        let action_entities = self
            .actions
            .iter()
            .map(|action_to_build| action_to_build.build(commands, actor_entity))
            .collect();

        commands
            .entity(actor_entity)
            .insert(Actor {
                actions: action_entities,
                current_path: VecDeque::new(),
            })
            .remove::<ActorBuilder>();

        self.marker_component.insert(commands, actor_entity);
    }
}

pub fn build_new_actor_system(
    mut commands: Commands,
    query: Query<(Entity, &ActorBuilder), Added<ActorBuilder>>,
    mut ev_request_plan: EventWriter<RequestPlanEvent>,
) {
    for (entity, actor_builder) in query.iter() {
        let actor_entity = commands.entity(entity).id();

        actor_builder.build(&mut commands, actor_entity);

        ev_request_plan.send(RequestPlanEvent(actor_entity));
    }
}
