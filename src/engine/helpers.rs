use std::sync::Arc;

use vulkano::{
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator,
        layout::DescriptorSetLayout,
    },
    device::Device,
    format::Format,
    image::{
        Image, ImageCreateInfo, ImageType, ImageUsage,
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::ImageView,
    },
    memory::allocator::{
        AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
    },
    pipeline::{
        ComputePipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    shader::ShaderModule,
};

use super::DescriptorResource;

pub fn create_pipeline(
    shader: Arc<ShaderModule>,
    entry_point: &str,
    device: Arc<Device>,
) -> Arc<ComputePipeline> {
    let compute_shader = shader.entry_point(entry_point).unwrap();

    let stage = PipelineShaderStageCreateInfo::new(compute_shader);

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout),
    )
    .unwrap()
}

pub fn create_image(
    image_type: ImageType,
    extent: [u32; 3],
    format: Format,
    usage: ImageUsage,
    mem_filter: MemoryTypeFilter,
    allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
) -> (Arc<Image>, Arc<ImageView>) {
    let image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            image_type,
            format,
            extent,
            usage,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: mem_filter,
            ..Default::default()
        },
    )
    .unwrap();

    let view = ImageView::new_default(image.clone()).unwrap();

    (image, view)
}

pub fn create_image_with_mipmap(
    image_type: ImageType,
    extent: [u32; 3],
    format: Format,
    usage: ImageUsage,
    mem_filter: MemoryTypeFilter,
    allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
) -> (Arc<Image>, Arc<ImageView>) {
    let mip_levels = (extent[0] as f32).log2().floor() as u32 + 1;

    let image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            image_type,
            format,
            extent,
            usage,
            mip_levels,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: mem_filter,
            ..Default::default()
        },
    )
    .unwrap();

    let view = ImageView::new_default(image.clone()).unwrap();

    (image, view)
}

pub fn create_image_array(
    image_type: ImageType,
    extent: [u32; 3],
    format: Format,
    usage: ImageUsage,
    mem_filter: MemoryTypeFilter,
    array_layers: u32,
    allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
) -> (Arc<Image>, Arc<ImageView>) {
    let image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            image_type,
            format,
            extent,
            usage,
            array_layers,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: mem_filter,
            ..Default::default()
        },
    )
    .unwrap();

    let view = ImageView::new_default(image.clone()).unwrap();

    (image, view)
}

pub fn create_image_array_with_mipmap(
    image_type: ImageType,
    extent: [u32; 3],
    format: Format,
    usage: ImageUsage,
    mem_filter: MemoryTypeFilter,
    array_layers: u32,
    allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
) -> (Arc<Image>, Arc<ImageView>) {
    let mip_levels = (extent[0] as f32).log2().floor() as u32 + 1;

    let image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            image_type,
            format,
            extent,
            usage,
            array_layers,
            mip_levels,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: mem_filter,
            ..Default::default()
        },
    )
    .unwrap();

    let view = ImageView::new_default(image.clone()).unwrap();

    (image, view)
}

pub fn create_sampler(
    mag_filter: Filter,
    min_filter: Filter,
    mipmap_mode: SamplerMipmapMode,
    address_mode: [SamplerAddressMode; 3],
    device: Arc<Device>,
    lod: Option<f32>,
    anisotropy: Option<f32>,
) -> Arc<Sampler> {
    Sampler::new(
        device.clone(),
        SamplerCreateInfo {
            mag_filter,
            min_filter,
            mipmap_mode,
            address_mode,
            lod: 0f32..=lod.unwrap_or(0f32),
            anisotropy,
            ..Default::default()
        },
    )
    .unwrap()
}

pub fn get_layout(pipeline: Arc<ComputePipeline>, index: usize) -> Arc<DescriptorSetLayout> {
    pipeline.layout().set_layouts().get(index).unwrap().clone()
}

pub fn create_descriptor_set(
    allocator: Arc<StandardDescriptorSetAllocator>,
    layout: Arc<DescriptorSetLayout>,
    data: &[(u32, DescriptorResource)],
) -> Arc<DescriptorSet> {
    let writes: Vec<WriteDescriptorSet> = data
        .iter()
        .map(|(binding, resource)| resource.clone().into_write(*binding))
        .collect();

    DescriptorSet::new(allocator.clone(), layout.clone(), writes, []).unwrap()
}
