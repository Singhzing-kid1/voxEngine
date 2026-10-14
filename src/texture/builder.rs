use rapier3d::parry::query::gjk::GJKResult::NoIntersection;
use vulkano::{format::Format, image::{ImageType, ImageUsage, sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode}, view::ImageView}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, sync::GpuFuture};

use vulkano::{
        buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, ClearColorImageInfo, CommandBufferUsage, CopyBufferToImageInfo,
        CopyImageInfo, PrimaryCommandBufferAbstract,
    },
};

use std::{ops::Add, ptr::from_mut, sync::Arc};

use crate::{common::Builder, engine::Engine};

use super::{Texture, Atlas};

#[derive(Default)]
pub struct AtlasBuilder {
    atlas_format: Option<Format>,

    atlas_extent: Option<[u32; 2]>,

    layers: Option<u32>,

    raw_textures: Option<Vec<Texture>>,
}

impl AtlasBuilder {
    pub fn add_image(mut self, path: impl Into<String>) -> Self {
        self.raw_textures.get_or_insert_with(Vec::new).push(Texture::new(path.into()));
        self.layers = Some(self.layers.get_or_insert(0).add(1));
        self
    }

    pub fn set_format(mut self, format: Format) -> Self {
        self.atlas_format = Some(format);
        self
    }

    pub fn set_atlas_resolution(mut self, resolution: [u32; 2]) -> Self {
        self.atlas_extent = Some(resolution);
        self
    }
}

impl Builder for AtlasBuilder {
    type Output = Atlas;
    type Error = &'static str;

    fn build(self, engine: &mut Engine) -> Result<Self::Output, Self::Error> {
        let mut atlas_data: Vec<u8> = Vec::new();

        let textures = self.raw_textures.unwrap();
        let extent = self.atlas_extent.unwrap();

        for texture in textures.iter() {
            if texture.get_image_dimensions() != &extent {
                return Err("all images must match the Atlas resolution");
            }

            atlas_data.extend_from_slice(texture.get_image_data());
        }

        let (atlas_image, atlas) = Engine::create_image_array(
            ImageType::Dim2d,
            [self.atlas_extent.unwrap()[0], self.atlas_extent.unwrap()[1], 1],
            self.atlas_format.unwrap(),
            ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            MemoryTypeFilter::PREFER_DEVICE,
            self.layers.unwrap(),
            engine.get_memory_allocator()
        );

        let atlas_sampler = Sampler::new(
            engine.get_device(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode: [SamplerAddressMode::Repeat; 3],
                mip_lod_bias: 0.0,
                ..Default::default()
            }
        ).unwrap();

        let mut atlas_uploads = AutoCommandBufferBuilder::primary(
            engine.get_command_buffer_allocator(),
            engine.get_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let atlas_staging_buffer = Buffer::from_iter(
            engine.get_memory_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            atlas_data
        )
        .unwrap();

        atlas_uploads
        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(atlas_staging_buffer, atlas_image)).unwrap();

        let _ = atlas_uploads
        .build()
        .unwrap()
        .execute(engine.get_queue())
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

        Ok (Atlas {
            atlas,
            atlas_sampler,
            atlas_format: self.atlas_format.unwrap(),
            layers: self.layers.unwrap(),
            raw_textures: textures
        })
        
    }
}
