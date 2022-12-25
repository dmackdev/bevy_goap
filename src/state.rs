use std::{any::TypeId, collections::HashMap, hash::Hash};

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
