use std::collections::VecDeque;

use bevy::prelude::{Entity, EventReader, Query};

use crate::{action::ActionState, actor::Actor};

pub struct RequestPlanEvent(pub(crate) Entity);

pub fn request_plan_event_handler_system(
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
