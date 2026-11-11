use std::fmt::Debug;
use std::path::PathBuf;
use vulkano::buffer::BufferContents;

use rapier3d::glamx::Vec3;

use crate::{
    engine::Engine,
    physics::{Physics, entity::Entity},
};

pub const TICK_RATE: f32 = 1.0 / 60.0; // 1 / fps -> matches rapier internals
pub const MAX_FRAME_TIME: f32 = 0.25;

#[repr(C)]
#[derive(BufferContents, Clone, Copy)]
pub struct LocalLight {
    pub pos_radius: [f32; 4],
    pub color_intensity: [f32; 4],
}

#[derive(Clone, Copy)]
pub struct GlobalLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

pub trait Updateable {
    fn fixed_update(&mut self, _physics: &mut Physics) {}

    fn update(&mut self, _alpha: f32, _physics: &mut Physics) {}
}

pub trait HasEntity: Debug {
    fn entity(&self) -> &Entity;
    fn entity_mut(&mut self) -> &mut Entity;
}

pub trait Builder {
    type Output;
    type Error;
    fn build(self, _engine: &mut Engine) -> Result<Self::Output, Self::Error>;
}

pub trait Buildable: Sized {
    type Builder: Builder<Output = Self> + Default;

    fn builder() -> Self::Builder {
        Self::Builder::default()
    }
}

pub trait AABB {
    fn aabb(&self) -> [Vec3; 2];
    fn gen_aabb(&self, position: Vec3) -> [Vec3; 2];
}
