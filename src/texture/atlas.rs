use super::texture::Texture;
use crate::engine::{DescriptorResource, Engine, helpers};
use getset::{CloneGetters, CopyGetters};
use rapier3d::glamx::{UVec2, Vec2};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, BufferImageCopy, CommandBufferUsage, CopyBufferToImageInfo,
        PrimaryCommandBufferAbstract,
    },
    descriptor_set::{DescriptorSet, layout::DescriptorSetLayout},
    format::Format,
    image::{
        ImageAspects, ImageSubresourceLayers, ImageType, ImageUsage,
        sampler::{Filter, LOD_CLAMP_NONE, Sampler, SamplerAddressMode, SamplerMipmapMode},
        view::ImageView,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    sync::GpuFuture,
};

#[derive(Clone, CopyGetters, CloneGetters)]
pub struct Atlas {
    atlas: HashMap<String, (Arc<ImageView>, Option<Arc<DescriptorSet>>)>,
    #[getset(get_clone = "pub with_prefix")]
    sampler: Arc<Sampler>,

    #[getset(get_copy = "pub with_prefix")]
    lod_bias: f32,
    format: Format,

    texture_count: u32,
}

impl Atlas {
    pub fn new(format: Format, lod_bias: f32, engine: &mut Engine) -> Self {
        let atlas: HashMap<String, (Arc<ImageView>, Option<Arc<DescriptorSet>>)> = HashMap::new();
        let sampler = helpers::create_sampler(
            Filter::Nearest,
            Filter::Linear,
            SamplerMipmapMode::Linear,
            [SamplerAddressMode::ClampToEdge; 3],
            engine.get_device(),
            Some(LOD_CLAMP_NONE),
            Some(8.0),
        );

        let texture_count = 0u32;

        Self {
            atlas,
            sampler,

            lod_bias,
            format,

            texture_count,
        }
    }

    pub fn add_image(
        &mut self,
        path: PathBuf,
        engine: &mut Engine,
        create_set: bool,
        layout: Option<Arc<DescriptorSetLayout>>,
    ) -> Self {
        let texture = Texture::new(path.clone());

        let (image, view) = helpers::create_image_with_mipmap(
            ImageType::Dim2d,
            [
                texture.get_image_dimensions()[0],
                texture.get_image_dimensions()[1],
                1,
            ],
            self.format,
            ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
        );

        let descriptor_set = if create_set && layout.is_some() {
            let set_entries = vec![(
                0,
                DescriptorResource::SampleTex(view.clone(), self.sampler.clone()),
            )];

            Some(helpers::create_descriptor_set(
                engine.get_descriptor_set_allocator(),
                layout.unwrap(),
                &set_entries,
            ))
        } else {
            None
        };

        let mip_levels = image.mip_levels();
        let (mut w, mut h) = (
            texture.get_image_dimensions()[0] as usize,
            texture.get_image_dimensions()[1] as usize,
        );

        let mut staging: Vec<u8> = Vec::new();
        let mut current = texture.get_image_data().to_vec();

        let mut regions: Vec<BufferImageCopy> = Vec::new();
        for level in 0..mip_levels {
            regions.push(BufferImageCopy {
                buffer_offset: staging.len() as u64,
                image_subresource: ImageSubresourceLayers {
                    aspects: ImageAspects::COLOR,
                    mip_level: level,
                    array_layers: 0..1,
                },
                image_extent: [w as u32, h as u32, 1],
                ..Default::default()
            });

            staging.extend_from_slice(&current);

            if level + 1 < mip_levels {
                let (next, new_width, new_height) = Atlas::downsample(&current, w, h);
                current = next;
                w = new_width;
                h = new_height;
            }
        }

        let mut uploads = AutoCommandBufferBuilder::primary(
            engine.get_command_buffer_allocator(),
            engine.get_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let staging_buffer = Buffer::from_iter(
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
            staging,
        )
        .unwrap();

        uploads
            .copy_buffer_to_image(CopyBufferToImageInfo {
                regions: regions.into(),
                ..CopyBufferToImageInfo::buffer_image(staging_buffer, image)
            })
            .unwrap();

        let _ = uploads
            .build()
            .unwrap()
            .execute(engine.get_queue())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        self.atlas
            .entry(path.into_string().unwrap())
            .or_insert((view, descriptor_set));

        self.clone()
    }

    pub fn add_image_from_sprite_sheet(
        &mut self,
        path: PathBuf,
        sprite_size: UVec2,
        texture_size: UVec2,
        engine: &mut Engine,
        create_set: bool,
        layout: Option<Arc<DescriptorSetLayout>>,
    ) -> Self {
        let mut textures: Vec<Texture> = Vec::new();
        let mut layers = 0u32;

        for column in 0..texture_size.x {
            for row in 0..texture_size.y {
                let idx = [column, row];

                textures.push(Texture::from_sprite_sheet(
                    path.clone(),
                    idx,
                    sprite_size.to_array(),
                ));
                layers += 1u32;
            }
        }

        let (image, view) = helpers::create_image_array_with_mipmap(
            ImageType::Dim2d,
            sprite_size.extend(1).to_array(),
            self.format,
            ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            MemoryTypeFilter::PREFER_DEVICE,
            layers,
            engine.get_memory_allocator(),
        );

        let descriptor_set = if create_set && layout.is_some() {
            let set_entries = vec![(
                0,
                DescriptorResource::SampleTex(view.clone(), self.sampler.clone()),
            )];

            Some(helpers::create_descriptor_set(
                engine.get_descriptor_set_allocator(),
                layout.unwrap(),
                &set_entries,
            ))
        } else {
            None
        };

        let mut staging: Vec<u8> = Vec::new();
        let mut current: Vec<Vec<u8>> = textures
            .iter()
            .map(|t| t.get_image_data().to_vec())
            .collect();

        let (mut w, mut h) = (image.extent()[0] as usize, image.extent()[1] as usize);

        let mip_levels = image.mip_levels();

        let mut regions: Vec<BufferImageCopy> = Vec::new();

        for level in 0..mip_levels {
            regions.push(BufferImageCopy {
                buffer_offset: staging.len() as u64,
                image_subresource: ImageSubresourceLayers {
                    aspects: ImageAspects::COLOR,
                    mip_level: level,
                    array_layers: 0..layers,
                },
                image_extent: [w as u32, h as u32, 1],
                ..Default::default()
            });

            for layer in &current {
                staging.extend_from_slice(layer);
            }

            if level + 1 < mip_levels {
                let mut next: Vec<Vec<u8>> = Vec::with_capacity(current.len());
                let (mut new_width, mut new_height) = (w, h);

                for layer in &current {
                    let (data, local_width, local_height) = Atlas::downsample(layer, w, h);
                    next.push(data);
                    new_width = local_width;
                    new_height = local_height;
                }

                current = next;
                w = new_width;
                h = new_height;
            }
        }

        let mut uploads = AutoCommandBufferBuilder::primary(
            engine.get_command_buffer_allocator(),
            engine.get_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let staging_buffer = Buffer::from_iter(
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
            staging,
        )
        .unwrap();

        uploads
            .copy_buffer_to_image(CopyBufferToImageInfo {
                regions: regions.into(),
                ..CopyBufferToImageInfo::buffer_image(staging_buffer, image)
            })
            .unwrap();

        let _ = uploads
            .build()
            .unwrap()
            .execute(engine.get_queue())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        self.atlas
            .entry(path.into_string().unwrap())
            .or_insert((view, descriptor_set));

        self.clone()
    }

    pub fn get_texture(&self, key: String) -> (Arc<ImageView>, Option<Arc<DescriptorSet>>) {
        self.atlas
            .get(&key)
            .unwrap_or_else(|| panic!("no entry for key: {key:?}"))
            .clone()
    }

    fn downsample(src: &[u8], w: usize, h: usize) -> (Vec<u8>, usize, usize) {
        let new_width = (w / 2).max(1);
        let new_height = (h / 2).max(1);
        let mut output = vec![0u8; new_width * new_height * 4];

        for y in 0..new_height {
            for x in 0..new_width {
                for channel in 0..4 {
                    let mut sum = 0u32;
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let source_x = (x * 2 + dx).min(w - 1);
                            let source_y = (y * 2 + dy).min(h - 1);
                            sum += src[(source_y * w + source_x) * 4 + channel] as u32;
                        }
                    }
                    output[(y * new_width + x) * 4 + channel] = ((sum + 2) / 4) as u8;
                }
            }
        }
        (output, new_width, new_height)
    }
}
