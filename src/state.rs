use std::{any::TypeId, collections::HashMap};

use bevy::prelude::{Commands, Component};

#[derive(Component)]
pub struct GoapWorldState {
    pub state: HashMap<TypeId, bool>,
}

impl GoapWorldState {
    fn new() -> Self {
        GoapWorldState {
            state: HashMap::new(),
        }
    }
}

pub fn create_world_state_system(mut commands: Commands) {
    commands.spawn_empty().insert(GoapWorldState::new());
}
