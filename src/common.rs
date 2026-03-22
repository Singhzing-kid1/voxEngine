use crate::entity::Entity;

pub trait Updateable {
    fn update(&mut self, delta_time: u128);
}

pub trait HasEntity {
    fn entity(&self) -> &Entity;
    fn entity_mut(&mut self) -> &mut Entity;
}
