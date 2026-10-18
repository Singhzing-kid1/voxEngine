use vulkano::{
    format::Format, image::{
        ImageType, ImageUsage, sampler::{Filter, LOD_CLAMP_NONE, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
    }, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, sync::GpuFuture,
};

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo,
        PrimaryCommandBufferAbstract,
    },
};

use vulkano::{
    command_buffer::{
  BufferImageCopy,
    },
    image::{
        ImageAspects, ImageSubresourceLayers,
    },
};

use std::ops::Add;

use crate::{common::Builder, engine::Engine};

use super::{Atlas, Texture};

#[derive(Default)]
pub struct AtlasBuilder {
    atlas_format: Option<Format>,

    atlas_extent: Option<[u32; 2]>,

    atlas_lod_bias: Option<f32>,

    layers: Option<u32>,

    raw_textures: Option<Vec<Texture>>,
}

impl AtlasBuilder {
    pub fn add_image(mut self, path: impl Into<String>) -> Self {
        self.raw_textures
            .get_or_insert_with(Vec::new)
            .push(Texture::new(path.into()));
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

    pub fn set_lod_bias(mut self, lod_bias: f32) -> Self {
        self.atlas_lod_bias = Some(lod_bias);
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

        let (atlas_image, atlas) = Engine::create_image_array_with_mipmap(
            ImageType::Dim2d,
            [
                self.atlas_extent.unwrap()[0],
                self.atlas_extent.unwrap()[1],
                1,
            ],
            self.atlas_format.unwrap(),
            ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::SAMPLED,
            MemoryTypeFilter::PREFER_DEVICE,
            self.layers.unwrap(),
            engine.get_memory_allocator(),
        );
        

        let layer_count = self.layers.unwrap();
        let mip_levels = atlas_image.mip_levels();          // grab before atlas_image is moved
        let (mut w, mut h) = (extent[0] as usize, extent[1] as usize);

        // current level, one Vec per layer
        let mut cur: Vec<Vec<u8>> = textures.iter().map(|t| t.get_image_data().to_vec()).collect();

        let mut regions: Vec<BufferImageCopy> = Vec::new();

        for level in 0..mip_levels {
            regions.push(BufferImageCopy {
                buffer_offset: atlas_data.len() as u64,
                image_subresource: ImageSubresourceLayers {
                    aspects: ImageAspects::COLOR,
                    mip_level: level,
                    array_layers: 0..layer_count,
                },
                image_extent: [w as u32, h as u32, 1],
                ..Default::default()
            });
            for layer in &cur {
                atlas_data.extend_from_slice(layer);               
            }
            if level + 1 < mip_levels {
                let mut next = Vec::with_capacity(cur.len());
                let (mut nw, mut nh) = (w, h);
                for layer in &cur {
                    let (d, a, b) = {
                            let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
                            let mut out = vec![0u8; nw * nh * 4];
                            for y in 0..nh {
                                for x in 0..nw {
                                    for c in 0..4 {
                                        let mut sum = 0u32;
                                        for dy in 0..2 {
                                            for dx in 0..2 {
                                                let sx = (x * 2 + dx).min(w - 1);
                                                let sy = (y * 2 + dy).min(h - 1);
                                                sum += layer[(sy * w + sx) * 4 + c] as u32;
                                            }
                                        }
                                        out[(y * nw + x) * 4 + c] = ((sum + 2) / 4) as u8;
                                    }
                                }
                            }
                            (out, nw, nh)
                    };
                    next.push(d);
                    nw = a;
                    nh = b;
                }
                cur = next;
                w = nw;
                h = nh;
            }
        }

        let atlas_sampler = Sampler::new(
            engine.get_device(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                lod: 0.0..=LOD_CLAMP_NONE,
                anisotropy: Some(8.0),
                ..Default::default()
            },
        )
        .unwrap();

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
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            atlas_data,
        )
        .unwrap();


        atlas_uploads.copy_buffer_to_image(CopyBufferToImageInfo {
            regions: regions.into(),
            ..CopyBufferToImageInfo::buffer_image(atlas_staging_buffer, atlas_image)
        }).unwrap();

        let _ = atlas_uploads
            .build()
            .unwrap()
            .execute(engine.get_queue())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        Ok(Atlas {
            atlas,
            atlas_sampler,
            atlas_format: self.atlas_format.unwrap(),
            atlas_lod_bias: self.atlas_lod_bias.unwrap(),
            layers: self.layers.unwrap(),
            raw_textures: textures,
        })
    }
}
