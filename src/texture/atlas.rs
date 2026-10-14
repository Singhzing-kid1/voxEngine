use getset::CloneGetters;
use std::sync::Arc;
use vulkano::{
    format::Format,
    image::{sampler::Sampler, view::ImageView},
};

use super::AtlasBuilder;
use super::Texture;
use crate::common::Buildable;

#[derive(CloneGetters)]
pub struct Atlas {
    #[getset(get_clone = "pub with_prefix")]
    pub(super) atlas: Arc<ImageView>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) atlas_sampler: Arc<Sampler>,

    pub(super) atlas_format: Format,

    pub(super) layers: u32,

    pub(super) raw_textures: Vec<Texture>,
}

impl Buildable for Atlas {
    type Builder = AtlasBuilder;
}
