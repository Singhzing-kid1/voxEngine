use getset::{CloneGetters, CopyGetters};
use std::sync::Arc;
use vulkano::{
    format::Format,
    image::{sampler::Sampler, view::ImageView},
};

use dear_imgui_reflect::ImGuiReflect;

use super::{builder::AtlasBuilder, texture::Texture};
use crate::common::Buildable;

#[allow(unused)]
#[derive(ImGuiReflect, CloneGetters, CopyGetters)]
pub struct Atlas {
    #[imgui(skip)]
    #[getset(get_clone = "pub with_prefix")]
    pub(super) atlas: Arc<ImageView>,
    #[imgui(skip)]
    #[getset(get_clone = "pub with_prefix")]
    pub(super) atlas_sampler: Arc<Sampler>,

    #[imgui(slider, min = -4.0, max = 0.0)]
    #[getset(get_copy = "pub with_prefix")]
    pub(super) atlas_lod_bias: f32,

    #[imgui(skip)]
    pub(super) atlas_format: Format,

    pub(super) layers: u32,

    #[imgui(skip)]
    pub(super) raw_textures: Vec<Texture>,
}

impl Buildable for Atlas {
    type Builder = AtlasBuilder;
}
