use glam::Vec3;

use crate::entity::Entity;

pub trait Updateable {
    fn update(&mut self, delta_time: u128);
}

pub trait HasEntity {
    fn entity(&self) -> &Entity;
    fn entity_mut(&mut self) -> &mut Entity;
}

pub trait AABB {
    fn aabb(&self) -> [Vec3; 2];   
}
