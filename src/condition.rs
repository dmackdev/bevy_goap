use bevy::ecs::system::SystemParam;

pub trait Condition {}

pub trait WorldCondition: SystemParam {
    fn update(&mut self);
}
