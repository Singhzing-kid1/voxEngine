use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage,
    },
    swapchain::{
        self, SwapchainPresentInfo,
    },
    sync::{GpuFuture},
};

use super::{Engine, Frame};

impl Engine {
    pub fn start_frame(&mut self) -> Frame {
        let (image_index, _, acquire_future) =
            swapchain::acquire_next_image(self.swapchain.clone(), None).unwrap();

        self.current_image_index = image_index;
        let swapchain_image = self.images[image_index as usize].clone();

        let builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        Frame {
            builder,
            swapchain_image,
            acquire_future,
        }
    }

    pub fn finish_frame(&mut self, frame: Frame) {
        let command_buffer = frame.builder.build().unwrap();

        let previous_future = self
            .previous_future
            .take()
            .unwrap_or_else(|| Box::new(vulkano::sync::now(self.device.clone())));

        let future = previous_future
            .join(frame.acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .boxed_send_sync();

        self.previous_future = Some(Box::new(future));
        if let Some(f) = &mut self.previous_future {
            f.flush().unwrap();
        }
    }

    pub fn present(&mut self) {
        let future = self.previous_future.take().unwrap();

        let present = future
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    self.current_image_index,
                ),
            )
            .then_signal_fence_and_flush()
            .unwrap();

        present.wait(None).unwrap();

        self.previous_future = Some(Box::new(present));

        if let Some(prev) = &mut self.previous_future {
            prev.as_mut().cleanup_finished();
        }
    }
}