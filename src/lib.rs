use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use bevy::prelude::{
    Added, Commands, Component, CoreStage, Entity, EventReader, EventWriter, Plugin, Query,
};

use inspector::GoapInspectorPlugin;

mod inspector;

pub struct GoapPlugin;

impl Plugin for GoapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(GoapInspectorPlugin)
            .add_event::<RequestPlanEvent>()
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

pub trait MarkerComponent: Send + Sync {
    fn insert(&self, commands: &mut Commands, entity_to_insert_to: Entity);
}

impl<T> MarkerComponent for T
where
    T: Component + Clone + Send + Sync,
{
    fn insert(&self, commands: &mut Commands, entity_to_insert_to: Entity) {
        commands.entity(entity_to_insert_to).insert(T::clone(self));
    }
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

#[derive(Component)]
pub struct Action {
    actor_entity: Entity,
    preconditions: HashMap<TypeId, bool>,
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

pub trait Condition {}
