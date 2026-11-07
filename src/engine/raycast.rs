use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage,
    },
    memory::allocator::{
        AllocationCreateInfo, MemoryTypeFilter,
    },
    pipeline::{
        Pipeline, PipelineBindPoint,
    },

    sync::{self, GpuFuture},
};

use crate::{common::RayHit,
    shader::raycast_shader::RayParams
};

use super::{Engine, DescriptorResource};

impl Engine {
    pub fn ray_hit_world(
        &self,
        origin: glam::Vec3,
        direction: glam::Vec3,
        ray_length: f32,
        resolution: [u32; 3],
    ) -> RayHit {
        let data = RayHit {
            hit: 0,
            distance: 0.0,
        };

        let result_buffer = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data,
        )
        .unwrap();

        let query_set_entries = vec![(0, DescriptorResource::RayBuffer(result_buffer.clone()))];

        let query_set = Engine::create_descriptor_set(
            self.descriptor_set_allocator.clone(),
            Engine::get_layout(self.raycast_compute_pipeline.clone(), 0),
            &query_set_entries,
        );

        let push_data = RayParams {
            origin: origin.to_array().into(),
            direction: direction.to_array(),
            max_distance: ray_length,
            voxel_resolution: resolution,
        };

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        unsafe {
            builder
                .bind_pipeline_compute(self.raycast_compute_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.raycast_compute_pipeline.layout().clone(),
                    0,
                    vec![self.voxel_set.as_ref().unwrap().clone(), query_set.clone()],
                )
                .unwrap()
                .push_constants(self.raycast_compute_pipeline.layout().clone(), 0, push_data)
                .unwrap()
                .dispatch([1, 1, 1])
                .unwrap();
        }

        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        let result = result_buffer.read().unwrap();

        RayHit {
            hit: result.hit,
            distance: result.distance,
        }
    }
}