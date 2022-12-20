use action::handle_completed_actions_system;
use actor::build_new_actor_system;
use bevy::prelude::{CoreStage, Plugin};

use inspector::GoapInspectorPlugin;
use planning::{request_plan_event_handler_system, RequestPlanEvent};

mod action;
mod actor;
mod common;
mod condition;
mod inspector;
mod planning;

pub use action::{Action, ActionState};
pub use actor::Actor;
pub use condition::Condition;

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
