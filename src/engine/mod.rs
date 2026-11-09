pub mod helpers;
pub mod scene;

mod camera;
mod debug;
mod engine;
mod event;
mod flags;
mod frame;
mod render;

//re-exports
pub use camera::Camera;
pub use debug::Debug;
pub use engine::Engine;
pub(crate) use flags::Flags;
pub use frame::Frame;

use crate::common::LocalLight;
use std::sync::Arc;
use vulkano::{
    buffer::Subbuffer,
    descriptor_set::WriteDescriptorSet,
    image::{sampler::Sampler, view::ImageView},
};

#[derive(Clone)]
pub enum DescriptorResource {
    ImageView(Arc<ImageView>),
    SampleTex(Arc<ImageView>, Arc<Sampler>),
    Sample(Arc<Sampler>),
    LightBuffer(Subbuffer<[LocalLight]>),
}

impl DescriptorResource {
    fn into_write(self, binding: u32) -> WriteDescriptorSet {
        match self {
            DescriptorResource::ImageView(view) => WriteDescriptorSet::image_view(binding, view),
            DescriptorResource::SampleTex(texture, sampler) => {
                WriteDescriptorSet::image_view_sampler(binding, texture, sampler)
            }
            DescriptorResource::Sample(sampler) => WriteDescriptorSet::sampler(binding, sampler),
            DescriptorResource::LightBuffer(buffer) => WriteDescriptorSet::buffer(binding, buffer),
        }
    }
}

pub trait Record {
    fn record(&mut self, frame: &mut Frame, engine: &mut Engine);
    fn record_overlay(&mut self, _frame: &mut Frame, _engine: &mut Engine) {}
    fn init(&mut self, engine: &mut Engine);
}
