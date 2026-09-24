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
    pub fn send_world_data(&mut self, world: &World) {
        let resolution = world.get_dimensions_as_arr();
        let max_height = world.get_dimensions().y;

        let (voxels, voxels_view) = Engine::create_image(
            ImageType::Dim3d,
            [resolution[0] / 4, resolution[1] / 4, resolution[2] / 8],
            Format::R32G32B32A32_UINT,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            self.memory_allocator.clone(),
        );

        let (biomes, biomes_view) = Engine::create_image(
            ImageType::Dim2d,
            [resolution[0], resolution[1], 1],
            Format::R8G8B8A8_UNORM,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            self.memory_allocator.clone(),
        );

        let voxel_staging_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            world.get_world_as_u32(),
        )
        .unwrap();

        let biomes_staging_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            world.get_biomes(),
        )
        .unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .clear_color_image(ClearColorImageInfo::image(voxels.clone()))
            .unwrap()
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                voxel_staging_buffer,
                voxels.clone(),
            ))
            .unwrap()
            .clear_color_image(ClearColorImageInfo::image(biomes.clone()))
            .unwrap()
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                biomes_staging_buffer,
                biomes.clone(),
            ))
            .unwrap();

        let _ = builder
            .build()
            .unwrap()
            .execute(self.queue.clone())
            .unwrap();

        let voxel_set_entries = vec![
            (0, DescriptorResource::ImageView(voxels_view)),
            (1, DescriptorResource::ImageView(biomes_view)),
        ];

        let voxel_set = Engine::create_descriptor_set(
            self.descriptor_set_allocator.clone(),
            Engine::get_layout(self.render_compute_pipeline.clone(), 1),
            &voxel_set_entries,
        );

        self.voxel_set = Some(voxel_set);
        self.max_fog_height = max_height * 0.33;
    }

    pub fn record_voxel_pass(&mut self, frame: &mut Frame, pixel_to_ray: Mat4, resolution: [u32; 3]) {
        let (resample_image, resample_view) = Engine::create_image(
            ImageType::Dim2d,
            [self.width as u32, self.height as u32, 1],
            self.image_format,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            self.memory_allocator.clone(),
        );

        let resample_set_entries = vec![
            (0, DescriptorResource::ImageView(self.view.clone())),
            (1, DescriptorResource::ImageView(resample_view)),
        ];

        let resample_set = Engine::create_descriptor_set(
            self.descriptor_set_allocator.clone(),
            Engine::get_layout(self.resample_compute_pipeline.clone(), 0),
            &resample_set_entries,
        );

        let push_data = render_compute_shader::PushConstants {
            pixelToRay: pixel_to_ray.to_cols_array_2d(),
            voxel_resolution: resolution,
            render_mode: self.current_render_mode as u32,
            max_ray_length: self.ray_length,
            max_height: self.max_fog_height,
        };

        let swapchain_image = frame.get_swapchain_image().clone();

        let builder = frame.get_builder_mut();

        unsafe {
            builder
                .clear_color_image(ClearColorImageInfo {
                    clear_value: ClearColorValue::Float([0.0, 0.0, 0.0, 1.0]),
                    ..ClearColorImageInfo::image(self.image.clone())
                })
                .unwrap()
                .bind_pipeline_compute(self.render_compute_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.render_compute_pipeline.layout().clone(),
                    0,
                    vec![
                        self.render_set.clone(),
                        self.voxel_set.as_ref().unwrap().clone(),
                    ],
                )
                .unwrap()
                .push_constants(self.render_compute_pipeline.layout().clone(), 0, push_data)
                .unwrap()
                .dispatch([
                    (self.width / self.render_scale) as u32 / 8,
                    (self.height / self.render_scale) as u32 / 8,
                    1,
                ])
                .unwrap()
                .bind_pipeline_compute(self.resample_compute_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.resample_compute_pipeline.layout().clone(),
                    0,
                    vec![resample_set.clone()],
                )
                .unwrap()
                .dispatch([self.width as u32 / 8, self.height as u32 / 8, 1])
                .unwrap()
                .copy_image(CopyImageInfo::images(
                    resample_image.clone(),
                    swapchain_image,
                ))
                .unwrap();
        };
    }

    pub fn record_clear(&mut self, frame: &mut Frame, color: Vec4) {
        let swapchain_image = frame.get_swapchain_image().clone();

        let builder = frame.get_builder_mut();

        builder.clear_color_image(ClearColorImageInfo {
            clear_value: ClearColorValue::Float([color.x, color.y, color.z, color.w]),
            ..ClearColorImageInfo::image(swapchain_image)
        }).unwrap();


    }

    pub fn record_blur(&mut self, frame: &mut Frame) {
        let (blur_pass_1, blur_pass_1_view) = Engine::create_image(
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