use std::sync::Arc;

use bevy::prelude::{Changed, Commands, Component, Entity, EventWriter, ParamSet, Query};

use crate::{
    actor::Actor, common::MarkerComponent, condition::Condition, planning::RequestPlanEvent,
    state::GoapState,
};

#[derive(Component, Debug)]
pub enum ActionState {
    /// An `Action` in this state is currently not executing, nor being considered for a plan.
    Idle,
    /// A plan has been requested for the `Actor` owning this `Action`, so this `Action` must be be evaluated for its viability to be included in the plan, and, if viable, to update the `Action` cost.
    ///
    /// This may depend on the current world state, extraneous to the local state of the `Actor`. It is important to note that since all possible `Actions` for a particular `Actor` are evaluated at the time of requesting the plan,
    /// any `Action` that could come later in a plan may no longer be satisfied by the world state at the time of starting it, so this may influence how you resolve the evaluation.
    ///
    /// - If the `Action` is viable, and a new cost can be calculated, update the cost and transition to `ActionState::EvaluationSuccess` to include it in planning considerations.
    ///
    /// - If the `Action` is not viable, or a new cost cannot be calculated, transition to `ActionState::EvaluationFailure` to exclude it from planning considerations.
    Evaluate,
    /// The `Action` has been evaluated and is deemed to be viable as a candidate in the next plan, with an updated cost.
    EvaluationSuccess,
    /// The `Action` has been evaluated and is deemed to be **not** viable as a candidate in the next plan. It will not be considered in the next plan.
    EvaluationFailure,
    /// The `Action` had a successful evaluation and was being considered for a plan, however the planner could not produce a plan. Use this state to perform any cleanup from operations that you might have performed during the `ActionState::Evaluate` phase, and transition back to `ActionState::Idle` when complete.
    PlanFailure,
    /// Use this state to perform initialisation operations required for the execution of the `Action`, e.g. update components on the `Actor`'s entity to move it to a target.
    Started,
    /// Use this state to check whether the `Action` has completed, e.g. check whether the `Actor` has reached a target.
    Executing,
    Complete,
    /// The `Action` failed during execution and the `Actor` requires a replan.
    Failure,
}

#[derive(Component, Clone, Debug)]
pub struct Action {
    pub actor_entity: Entity,
    pub(crate) preconditions: GoapState,
    pub(crate) postconditions: GoapState,
    pub(crate) cost: i32,
}

impl Action {
    pub fn build(marker_component: impl MarkerComponent + 'static) -> ActionBuilder {
        ActionBuilder {
            marker_component: Arc::new(marker_component),
            preconditions: GoapState::new(),
            postconditions: GoapState::new(),
        }
    }

    pub fn update_cost(&mut self, new_cost: u32) {
        self.cost = new_cost as i32;
    }
}

#[derive(Clone)]
pub struct ActionBuilder {
    marker_component: Arc<dyn MarkerComponent>,
    preconditions: GoapState,
    postconditions: GoapState,
}

impl ActionBuilder {
    pub fn with_precondition<T: Condition + 'static>(
        mut self,
        _precondition: T,
        value: bool,
    ) -> ActionBuilder {
        self.preconditions.insert::<T>(value);
        self
    }

    pub fn with_postcondition<T: Condition + 'static>(
        mut self,
        _postcondition: T,
        value: bool,
    ) -> ActionBuilder {
        self.postconditions.insert::<T>(value);
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
                postconditions: self.postconditions.clone(),
                cost: 1,
            })
            .insert(ActionState::Idle)
            .id();

        self.marker_component.insert(commands, action_entity);

        action_entity
    }
}

#[allow(clippy::type_complexity)]
pub fn action_system(
    mut actors: Query<&mut Actor>,
    mut set: ParamSet<(
        Query<(&Action, &mut ActionState), Changed<ActionState>>,
        Query<&mut ActionState>,
    )>,
    mut ev_request_plan: EventWriter<RequestPlanEvent>,
) {
    let mut changed_action_states_query = set.p0();

    if changed_action_states_query.iter().count() == 0 {
        return;
    }

    let mut completed = vec![];

    for (action, mut action_state) in changed_action_states_query.iter_mut() {
        match *action_state {
            ActionState::Complete => {
                *action_state = ActionState::Idle;

                completed.push((action.actor_entity, action.postconditions.clone()));
            }
            ActionState::Failure => {
                *action_state = ActionState::Idle;
                ev_request_plan.send(RequestPlanEvent(action.actor_entity));
            }
            _ => (),
        };
    }

    let mut all_action_states_query = set.p1();

    for (actor_entity, postconditions) in completed {
        let mut actor = actors.get_mut(actor_entity).unwrap();

        if let Some(next_action_entity) = actor.complete_action(postconditions) {
            let mut next_action_state = all_action_states_query
                .get_mut(*next_action_entity)
                .unwrap();
            *next_action_state = ActionState::Started;
        }
    }
}
