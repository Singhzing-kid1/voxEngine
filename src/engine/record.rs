use glam::{Mat4, Vec4, vec2};

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, ClearColorImageInfo, CommandBufferUsage,
        CopyBufferToImageInfo, CopyImageInfo,
        PrimaryCommandBufferAbstract,
    },
    format::{ClearColorValue, Format},
    image::{ImageType, ImageUsage, view::ImageView},
    memory::allocator::{
        AllocationCreateInfo, MemoryTypeFilter,
    },
    pipeline::{
        Pipeline, PipelineBindPoint,
    },
};

use crate::{world::World, shader::render_compute_shader, shader::pause_blur_shader};


use super::{Engine, DescriptorResource, Frame};

impl Engine {
    pub fn record_clear(&mut self, frame: &mut Frame, color: Vec4) {
        let swapchain_image = frame.get_swapchain_image().clone();

        let builder = frame.get_builder_mut();

        builder.clear_color_image(ClearColorImageInfo {
            clear_value: ClearColorValue::Float([color.x, color.y, color.z, color.w]),
            ..ClearColorImageInfo::image(swapchain_image)
        }).unwrap();


    }

    pub fn record_blur(&mut self, frame: &mut Frame) {
        let (_blur_pass_1, blur_pass_1_view) = Engine::create_image(
            ImageType::Dim2d,
            [self.width as u32, self.height as u32, 1],
            self.image_format,
            ImageUsage::STORAGE,
            MemoryTypeFilter::PREFER_DEVICE,
            self.memory_allocator.clone()
        );

        let (blur_final, blur_final_view) = Engine::create_image(
            ImageType::Dim2d,
            [self.width as u32, self.height as u32, 1],
            self.image_format,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC,
            MemoryTypeFilter::PREFER_DEVICE,
            self.memory_allocator.clone()
        );

        let swapchain_image = frame.get_swapchain_image().clone();

        let swapchain_view = ImageView::new_default(swapchain_image.clone()).unwrap();

        let blur_pass_1_entries = vec![
            (0, DescriptorResource::ImageView(swapchain_view.clone())),
            (1, DescriptorResource::ImageView(blur_pass_1_view.clone()))
        ];

        let blur_final_entries = vec![
            (0, DescriptorResource::ImageView(blur_pass_1_view.clone())),
            (1, DescriptorResource::ImageView(blur_final_view))
        ];

        let blur_pass_1_set = Engine::create_descriptor_set(self.descriptor_set_allocator.clone(), Engine::get_layout(self.pause_blur_compute_pipeline.clone(), 0), &blur_pass_1_entries);
        let blur_final_set = Engine::create_descriptor_set(self.descriptor_set_allocator.clone(), Engine::get_layout(self.pause_blur_compute_pipeline.clone(), 0),&blur_final_entries);

        let blur_pass_1_data = pause_blur_shader::PushConstants {
            direction: vec2(1.0, 0.0).into(),
            sigma: 4.0
        };

        let blur_final_data = pause_blur_shader::PushConstants {
            direction: vec2(0.0, 1.0).into(),
            sigma: 4.0
        };

        let builder = frame.get_builder_mut();

        unsafe {
            builder.bind_pipeline_compute(self.pause_blur_compute_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pause_blur_compute_pipeline.layout().clone(),
                0,
                vec![blur_pass_1_set.clone()]
            )
            .unwrap()
            .push_constants(self.pause_blur_compute_pipeline.layout().clone(), 0, blur_pass_1_data)
            .unwrap()
            .dispatch([
                self.width as u32 / 8,
                self.height as u32 / 8,
                1
            ])
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pause_blur_compute_pipeline.layout().clone(),
                0,
                vec![blur_final_set.clone()]
            )
            .unwrap()
            .push_constants(self.pause_blur_compute_pipeline.layout().clone(), 0, blur_final_data)
            .unwrap()
            .dispatch([
                self.width as u32 / 8,
                self.height as u32 / 8,
                1
            ])
            .unwrap()
            .copy_image(
                CopyImageInfo::images(
                    blur_final.clone(),
                    swapchain_image
                )
            )
            .unwrap();            
        }

    }
}