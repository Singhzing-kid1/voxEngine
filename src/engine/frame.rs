use getset::{Getters, MutGetters};

use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    image::Image,
    swapchain::SwapchainAcquireFuture,
};

#[derive(Getters, MutGetters)]
pub struct Frame {
    #[getset(get_mut = "pub with_prefix")]
    pub(super) builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    #[getset(get = "pub with_prefix")]
    pub(super) swapchain_image: Arc<Image>,

    pub(super) acquire_future: SwapchainAcquireFuture,
}

impl Frame {
    pub fn new(
        builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        swapchain_image: Arc<Image>,
        acquire_future: SwapchainAcquireFuture,
    ) -> Self {
        Self {
            builder,
            swapchain_image,
            acquire_future,
        }
    }
}
