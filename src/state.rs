use std::{any::TypeId, collections::HashMap};

use bevy::prelude::{Commands, Component};

use crate::WorldCondition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoapState {
    pub(crate) state: HashMap<TypeId, bool>,
}

impl GoapState {
    pub(crate) fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    pub(crate) fn insert<T: 'static>(&mut self, value: bool) {
        self.state.insert(TypeId::of::<T>(), value);
    }

    pub(crate) fn extend(&mut self, other: GoapState) {
        self.state.extend(other.state);
    }
}

#[derive(Component, Debug)]
pub struct GoapWorldState {
    state: GoapState,
}

impl GoapWorldState {
    fn new() -> Self {
        GoapWorldState {
            state: GoapState::new(),
        }
    }

    pub fn get(&self) -> GoapState {
        self.state.clone()
    }

    pub fn insert<T: WorldCondition + 'static>(&mut self, value: bool) {
        self.state.insert::<T>(value);
    }
}

pub fn create_world_state_system(mut commands: Commands) {
    commands.spawn_empty().insert(GoapWorldState::new());
}
