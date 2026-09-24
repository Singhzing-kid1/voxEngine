use std::sync::Arc;

use vulkano::{
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator,
        layout::DescriptorSetLayout,
    },
    device::{
        Device
    },
    format::{Format},
    image::{Image, ImageCreateInfo, ImageType, ImageUsage, view::ImageView},
    memory::allocator::{
        AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
    },
    pipeline::{
        ComputePipeline, Pipeline, PipelineLayout,
        PipelineShaderStageCreateInfo, compute::ComputePipelineCreateInfo,
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    shader::ShaderModule,
};

use super::{Engine, DescriptorResource};

impl Engine {
    pub fn get_dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn get_hardware_info(&self) -> String {
        let props = self.device.physical_device().properties();
        format!("{}\n{}", props.device_name, props.api_version)
    }

    pub fn get_frame_rate(&self) -> u64 {
        (1000.0 / self.delta_time as f64).round() as u64
    }

    pub fn start_text_input(&self) {
        self.sdl_context
            .video()
            .unwrap()
            .text_input()
            .start(&self.window);
    }

    pub fn stop_text_input(&self) {
        self.sdl_context
            .video()
            .unwrap()
            .text_input()
            .stop(&self.window);
    }

    pub(super) fn create_pipeline(
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

    pub(super) fn create_image(
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

    pub(super) fn get_layout(pipeline: Arc<ComputePipeline>, index: usize) -> Arc<DescriptorSetLayout> {
        pipeline.layout().set_layouts().get(index).unwrap().clone()
    }

    pub(super) fn create_descriptor_set(
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
}