use action::action_system;
use actor::{actor_state_system, build_new_actor_system};
use bevy::prelude::{CoreStage, IntoSystemDescriptor, Plugin, SystemSet};

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
        app.add_event::<RequestPlanEvent>()
            .add_startup_system(create_planning_state)
            .add_system(build_new_actor_system)
            .add_system(action_system)
            .add_system(actor_state_system.after(action_system))
            .add_system_set_to_stage(
                CoreStage::Last,
                SystemSet::new()
                    .with_system(request_plan_event_handler_system)
                    .with_system(create_plan_system.after(request_plan_event_handler_system)),
            );
    }
}
