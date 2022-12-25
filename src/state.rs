use std::{any::TypeId, collections::HashMap, hash::Hash};

use bevy::prelude::{Commands, Component};

use crate::WorldCondition;

#[derive(Debug, Clone, Eq)]
pub struct GoapState {
    pub(crate) state: HashMap<TypeId, bool>,
}

impl PartialEq for GoapState {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl Hash for GoapState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (key, value) in self.state.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
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
