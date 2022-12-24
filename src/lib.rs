use action::action_system;
use actor::build_new_actor_system;
use bevy::prelude::{CoreStage, IntoSystemDescriptor, Plugin, SystemSet};

use inspector::GoapInspectorPlugin;
use planning::{
    create_plan_system, create_planning_state, request_plan_event_handler_system, RequestPlanEvent,
};

mod action;
mod actor;
mod common;
mod condition;
mod inspector;
mod planning;
mod state;

pub use action::{Action, ActionState};
pub use actor::Actor;
pub use condition::{Condition, WorldCondition};
pub use state::GoapWorldState;

use state::create_world_state_system;

pub struct GoapPlugin;

impl Plugin for GoapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(GoapInspectorPlugin)
            .add_event::<RequestPlanEvent>()
            .add_startup_system(create_world_state_system)
            .add_startup_system(create_planning_state)
            .add_system(build_new_actor_system)
            .add_system(action_system)
            .add_system_set_to_stage(
                CoreStage::Last,
                SystemSet::new()
                    .with_system(request_plan_event_handler_system)
                    .with_system(create_plan_system.after(request_plan_event_handler_system)),
            );
    }
}
