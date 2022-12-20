use std::{collections::VecDeque, sync::Arc};

use bevy::prelude::{
    Added, Commands, Component, CoreStage, Entity, EventReader, EventWriter, Plugin, Query,
};

pub struct GoapPlugin;

impl Plugin for GoapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<RequestPlanEvent>()
            .add_system(build_new_actor_system)
            .add_system(handle_completed_actions_system)
            .add_system_to_stage(CoreStage::Last, request_plan_event_handler_system);
    }
}

#[derive(Component)]
pub struct Actor {
    actions: Vec<Entity>,
    current_path: VecDeque<Entity>,
}

impl Actor {
    pub fn build(marker_component: impl ActorMarkerComponent + 'static) -> ActorBuilder {
        ActorBuilder {
            marker_component: Arc::new(marker_component),
            actions: vec![],
        }
    }
}

#[derive(Component)]
pub struct ActorBuilder {
    marker_component: Arc<dyn ActorMarkerComponent>,
    actions: Vec<Arc<dyn ActionBuilder>>,
}

pub trait ActorMarkerComponent: Send + Sync {
    fn build(&self, commands: &mut Commands, actor_entity: Entity);
}

impl<T> ActorMarkerComponent for T
where
    T: Component + Clone + Send + Sync,
{
    fn build(&self, commands: &mut Commands, actor_entity: Entity) {
        commands.entity(actor_entity).insert(T::clone(self));
    }
}

impl ActorBuilder {
    pub fn with_action(mut self, action: impl ActionBuilder + 'static) -> Self {
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

        self.marker_component.build(commands, actor_entity);
    }
}

#[derive(Component)]
struct Action {
    actor_entity: Entity,
}

pub trait ActionBuilder: Send + Sync {
    fn build(&self, commands: &mut Commands, actor_entity: Entity) -> Entity;
}

impl<T> ActionBuilder for T
where
    T: Component + Clone + Send + Sync,
{
    fn build(&self, commands: &mut Commands, actor_entity: Entity) -> Entity {
        let action_component = T::clone(self);
        commands
            .spawn_empty()
            .insert(action_component)
            .insert(Action { actor_entity })
            .insert(ActionState::Idle)
            .id()
    }
}

fn build_new_actor_system(
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

fn request_plan_event_handler_system(
    mut ev_levelup: EventReader<RequestPlanEvent>,
    mut actors: Query<&mut Actor>,
    mut action_states: Query<&mut ActionState>,
) {
    for ev in ev_levelup.iter() {
        println!("Plan requested for {:?}", ev.0);

        if let Ok(mut actor) = actors.get_mut(ev.0) {
            println!("Updating path for actor");

            // TODO: Use pathfinding algorithm to determine path.
            actor.current_path = VecDeque::from_iter(actor.actions.clone().into_iter());

            if let Some(action_entity) = actor.current_path.front() {
                let mut action_state = action_states.get_mut(*action_entity).unwrap();
                *action_state = ActionState::Executing;
            }
        }
    }
}

fn handle_completed_actions_system(
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

#[derive(Component)]
pub enum ActionState {
    Idle,
    Executing,
    Complete,
}

struct RequestPlanEvent(Entity);
