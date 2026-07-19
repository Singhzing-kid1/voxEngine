use glam::Vec3;
use std::fmt::Debug;
use vulkano::buffer::BufferContents;

use crate::entity::Entity;

#[derive(BufferContents)]
#[repr(C)]
pub struct RayHit {
    pub hit: u32,
    pub distance: f32,
}

pub trait Updateable {
    fn update(&mut self, delta_time: u128);
}

pub trait HasEntity: Debug {
    fn entity(&self) -> &Entity;
    fn entity_mut(&mut self) -> &mut Entity;
}

pub trait AABB {
    fn aabb(&self) -> [Vec3; 2];   
    fn gen_aabb(&self, position: Vec3) -> [Vec3; 2];
}
