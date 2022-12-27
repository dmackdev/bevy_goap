use action::action_state_system;
use actor::{actor_state_system, build_new_actor_system};
use bevy::prelude::{CoreStage, IntoSystemDescriptor, Plugin, StageLabel, SystemSet, SystemStage};

use planning::{
    create_plan_system, create_planning_state, request_plan_event_handler_system, RequestPlanEvent,
};

mod action;
mod actor;
mod common;
mod condition;
mod planning;
mod state;

#[cfg(feature = "inspector")]
pub mod inspector;

pub use action::{Action, ActionState, EvaluationResult};
pub use actor::{Actor, ActorState};
pub use condition::Condition;

pub struct GoapPlugin;

impl Plugin for GoapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<RequestPlanEvent>();

        app.add_startup_system(create_planning_state);

        app.add_system_to_stage(CoreStage::First, build_new_actor_system);

        // User Action systems should be added to this stage, which can check for progress of Actions after typical user systems (e.g. for movement) complete during Update.
        app.add_stage_after(
            CoreStage::Update,
            GoapStage::Actions,
            SystemStage::parallel(),
        );

        // We add another stage for change detection of completed or failed actions, which may update the ActorState to reflect a completed or failed plan.
        app.add_stage_after(
            GoapStage::Actions,
            InternalGoapStage::ActionStateTransition,
            SystemStage::parallel(),
        );
        app.add_system_to_stage(
            InternalGoapStage::ActionStateTransition,
            action_state_system,
        );

        // User Actor systems should be added to this stage, which can react to an Actor's completed or failed plan.
        app.add_stage_after(
            InternalGoapStage::ActionStateTransition,
            GoapStage::Actors,
            SystemStage::parallel(),
        );

        // We add another stage for change detection of ActorStates for Actors that may require a new plan.
        app.add_stage_after(
            GoapStage::Actors,
            InternalGoapStage::ActorStateTransition,
            SystemStage::parallel(),
        );
        app.add_system_set_to_stage(
            InternalGoapStage::ActorStateTransition,
            SystemSet::new()
                .with_system(actor_state_system)
                .with_system(request_plan_event_handler_system.after(actor_state_system)),
        );

        app.add_system_to_stage(CoreStage::Last, create_plan_system);
    }
}

#[derive(StageLabel)]
pub enum GoapStage {
    /// User `Action` systems should be added to this stage.
    Actions,
    /// User `Actor` systems should be added to this stage.
    Actors,
}

#[derive(StageLabel)]
enum InternalGoapStage {
    /// Internal stage to react to changed `ActionState`s from user `Action` systems.
    ActionStateTransition,
    /// Internal stage to react to changed `ActorState`s from user `Actor` systems.
    ActorStateTransition,
}
