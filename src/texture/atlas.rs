use getset::{CloneGetters, CopyGetters};
use std::sync::Arc;
use vulkano::{
    format::Format,
    image::{sampler::Sampler, view::ImageView},
};

#[cfg(feature = "imgui")]
use dear_imgui_reflect::ImGuiReflect;

use super::{builder::AtlasBuilder, texture::Texture};
use crate::common::Buildable;

#[allow(unused)]
#[derive(CloneGetters, CopyGetters)]
#[cfg_attr(feature = "imgui", derive(ImGuiReflect))]
pub struct Atlas {
    #[cfg_attr(feature = "imgui", imgui(skip))]
    #[getset(get_clone = "pub with_prefix")]
    pub(super) atlas: Arc<ImageView>,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    #[getset(get_clone = "pub with_prefix")]
    pub(super) atlas_sampler: Arc<Sampler>,

    #[cfg_attr(feature = "imgui", imgui(slider, min = -4.0, max = 0.0))]
    #[getset(get_copy = "pub with_prefix")]
    pub(super) atlas_lod_bias: f32,

    #[cfg_attr(feature = "imgui", imgui(skip))]
    pub(super) atlas_format: Format,

    #[cfg_attr(feature = "imgui", imgui(skip))]
    pub(super) layers: u32,

    #[cfg_attr(feature = "imgui", imgui(skip))]
    pub(super) raw_textures: Vec<Texture>,
}

impl Buildable for Atlas {
    type Builder = AtlasBuilder;
}
