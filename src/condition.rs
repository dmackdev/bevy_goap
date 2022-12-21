pub trait Condition {}

pub trait WorldCondition {
    fn value(&self) -> bool;
}
