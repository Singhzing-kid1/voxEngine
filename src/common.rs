use std::fmt::Debug;
use std::path::PathBuf;
use vulkano::{buffer::BufferContents, padded::Padded};

use rapier3d::glamx::Vec3;

use crate::{
    engine::Engine,
    physics::{Physics, entity::Entity},
    shader::*,
};

pub(crate) const TICK_RATE: f32 = 1.0 / 60.0; // 1 / fps -> matches rapier internals
pub(crate) const MAX_FRAME_TIME: f32 = 0.25;

#[derive(BufferContents)]
#[repr(C)]
pub struct RayHit {
    pub hit: u32,
    pub distance: f32,
}

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

impl From<GlobalLight> for render_compute_shader::GlobalLight {
    fn from(sun: GlobalLight) -> Self {
        Self {
            position: Padded(sun.position.to_array()),
            color: sun.color.to_array(),
            intensity: sun.intensity,
        }
    }
}

impl From<GlobalLight> for apply_light_shader::GlobalLight {
    fn from(sun: GlobalLight) -> Self {
        Self {
            position: Padded(sun.position.to_array()),
            color: sun.color.to_array(),
            intensity: sun.intensity,
        }
    }
}

pub trait Updateable {
    fn fixed_update(&mut self, _physics: &mut Physics) {}

    fn update(&mut self, _alpha: f32, _physics: &mut Physics) {}
}

pub trait HasEntity: Debug {
    fn entity(&self) -> &Entity;
    fn entity_mut(&mut self) -> &mut Entity;
}

pub fn asset_path(path: &str) -> PathBuf {
    let rel = path.trim_start_matches(['/', '\\']);

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(rel);
            if candidate.exists() {
                return candidate;
            }
        }
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
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
