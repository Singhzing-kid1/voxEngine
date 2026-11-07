use std::sync::Arc;

use ash::vk::{
    AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
    ClearColorValue, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo,
    CommandBufferLevel, CommandBufferResetFlags, CommandBufferUsageFlags, CommandPool,
    CommandPoolCreateFlags, CommandPoolCreateInfo, ComponentMapping, DependencyFlags, Extent2D,
    Fence, FenceCreateFlags, FenceCreateInfo, Framebuffer, FramebufferCreateInfo, ImageAspectFlags,
    ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, ImageView, ImageViewCreateInfo,
    ImageViewType, Offset2D, PipelineBindPoint, PipelineStageFlags, QUEUE_FAMILY_IGNORED, Rect2D,
    RenderPass, RenderPassBeginInfo, RenderPassCreateInfo, SUBPASS_EXTERNAL, SampleCountFlags,
    SubmitInfo, SubpassContents, SubpassDependency, SubpassDescription,
};

use dear_imgui_ash::{AshRenderer, AshRendererConfig};
use dear_imgui_rs::Context;
use dear_imgui_sdl3::Sdl3PlatformBackend;

use vulkano::{
    VulkanObject,
    device::{Device, Queue},
    image::Image,
    instance::Instance,
    swapchain::Swapchain,
};

use dear_imgui_reflect::{ImGuiReflectExt, ReflectSession};

use crate::engine::Engine;
use crate::scene::Scene;

#[allow(unused)]
pub struct Debug {
    device: Arc<Device>,
    queue: Arc<Queue>,
    instance: Arc<Instance>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    command_buffers: Vec<CommandBuffer>,
    fences: Vec<Fence>,

    ash_device: Arc<ash::Device>,
    ash_instance: Arc<ash::Instance>,

    platform: Sdl3PlatformBackend,
    renderer: AshRenderer,
    context: Context,
    reflect: ReflectSession,

    render_pass: RenderPass,
    framebuffer: Vec<Framebuffer>,
    image_views: Vec<ImageView>,
    command_pool: CommandPool,
    getting_input: bool,
}

impl Debug {
    pub fn new(engine: &Engine) -> Self {
        let library = engine.get_library();
        let device = engine.get_device();
        let instance = engine.get_instance();
        let queue = engine.get_queue();
        let swapchain = engine.get_swapchain();
        let images = engine.get_images();

        let ash_instance = Arc::new(unsafe {
            let static_fn = ash::StaticFn::load(|name| {
                std::mem::transmute(
                    library.get_instance_proc_addr(instance.handle(), name.as_ptr()),
                )
            });
            ash::Instance::load(&static_fn, instance.handle())
        });

        let ash_device =
            Arc::new(unsafe { ash::Device::load(ash_instance.fp_v1_0(), device.handle()) });

        let mut context = Context::create();

        let platform = unsafe {
            Sdl3PlatformBackend::init_for_vulkan(&mut context, engine.get_window()).unwrap()
        };

        let command_pool = unsafe {
            ash_device
                .create_command_pool(
                    &CommandPoolCreateInfo::default()
                        .queue_family_index(queue.queue_family_index())
                        .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                    None,
                )
                .unwrap()
        };

        let command_buffers = unsafe {
            ash_device
                .allocate_command_buffers(
                    &CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .level(CommandBufferLevel::PRIMARY)
                        .command_buffer_count(images.len() as u32),
                )
                .unwrap()
        };

        let fences: Vec<Fence> = (0..images.len())
            .map(|_| unsafe {
                ash_device
                    .create_fence(
                        &FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED),
                        None,
                    )
                    .unwrap()
            })
            .collect();

        let format: ash::vk::Format = swapchain.image_format().into();

        let attachment = AttachmentDescription::default()
            .format(format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::STORE)
            .initial_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);

        let color_ref = AttachmentReference::default()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = SubpassDescription::default()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_ref));

        let dependency = SubpassDependency::default()
            .src_subpass(SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(PipelineStageFlags::TRANSFER)
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(
                AccessFlags::COLOR_ATTACHMENT_READ | AccessFlags::COLOR_ATTACHMENT_WRITE,
            );

        let render_pass = unsafe {
            ash_device
                .create_render_pass(
                    &RenderPassCreateInfo::default()
                        .attachments(std::slice::from_ref(&attachment))
                        .subpasses(std::slice::from_ref(&subpass))
                        .dependencies(std::slice::from_ref(&dependency)),
                    None,
                )
                .unwrap()
        };

        let extent = swapchain.image_extent();

        let image_views: Vec<ImageView> = images
            .iter()
            .map(|image| unsafe {
                ash_device
                    .create_image_view(
                        &ImageViewCreateInfo::default()
                            .image(image.handle())
                            .view_type(ImageViewType::TYPE_2D)
                            .format(format)
                            .components(ComponentMapping::default())
                            .subresource_range(ImageSubresourceRange {
                                aspect_mask: ImageAspectFlags::COLOR,
                                base_mip_level: 0,
                                level_count: 1,
                                base_array_layer: 0,
                                layer_count: 1,
                            }),
                        None,
                    )
                    .unwrap()
            })
            .collect();

        let framebuffer = image_views
            .iter()
            .map(|view| unsafe {
                ash_device
                    .create_framebuffer(
                        &FramebufferCreateInfo::default()
                            .render_pass(render_pass)
                            .attachments(std::slice::from_ref(view))
                            .width(extent[0])
                            .height(extent[1])
                            .layers(1),
                        None,
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let config = AshRendererConfig::with_render_pass(
            (*ash_device).clone(),
            queue.handle(),
            command_pool,
            render_pass,
        );

        let renderer = unsafe {
            AshRenderer::with_default_allocator(
                &ash_instance,
                device.physical_device().handle(),
                config,
                &mut context,
            )
            .unwrap()
        };

        Debug {
            device,
            queue,
            instance,
            swapchain,
            images,
            command_buffers,
            fences,

            ash_device,
            ash_instance,

            platform,
            renderer,
            context,
            reflect: ReflectSession::new(),

            render_pass,
            framebuffer,
            image_views,
            command_pool,
            getting_input: false,
        }
    }

    pub fn render(&mut self, engine: &mut Engine, scene: &mut dyn Scene) {
        let image_index = engine.get_current_image_index() as usize;

        for event in engine.get_collected_events() {
            self.platform
                .process_event(&mut self.context, &event)
                .unwrap();
        }

        let wants_text = self.context.io().want_text_input();
        if wants_text && !self.getting_input {
            engine.start_text_input();
            self.getting_input = true;
        } else if !wants_text && self.getting_input {
            engine.stop_text_input();
            self.getting_input = false;
        }

        self.platform.new_frame(&mut self.context).unwrap();

        let frame = self.context.begin_frame();
        {
            let ui = frame.ui();
            let reflect = &self.reflect;
            ui.window("Debug - Gameplay")
                .size([500.0, 500.0], dear_imgui_rs::Condition::FirstUseEver)
                .build(|| {
                    ui.text(engine.get_hardware_info());
                    ui.text(engine.get_frame_rate().to_string());

                    let mut inspector = ui.inspector(reflect);
                    inspector.input("Flags", engine.get_flags_mut());
                    inspector.input("Render Mode", engine.get_current_render_mode_mut());

                    scene.imgui_reflect_dyn(&mut inspector);
                });
        }

        let pending = frame.render(self.renderer.renderer_consumer().unwrap());
        let (reconciled, retirement) = self.renderer.prepare_frame(pending).unwrap();

        if let Some(batch) = retirement {
            self.renderer.wait_for_texture_retirements(batch).unwrap();
        }

        let command_buffer = self.command_buffers[image_index];
        let fence = self.fences[image_index];

        unsafe {
            self.ash_device
                .wait_for_fences(&[fence], true, u64::MAX)
                .unwrap();
            self.ash_device.reset_fences(&[fence]).unwrap();
            self.ash_device
                .reset_command_buffer(command_buffer, CommandBufferResetFlags::empty())
                .unwrap();
        }

        unsafe {
            self.ash_device
                .begin_command_buffer(
                    command_buffer,
                    &CommandBufferBeginInfo::default()
                        .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .unwrap();

            let clear_value = ClearValue {
                color: ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            };

            let extent = self.swapchain.image_extent();
            let render_pass_begin = RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffer[image_index])
                .render_area(Rect2D {
                    offset: Offset2D { x: 0, y: 0 },
                    extent: Extent2D {
                        width: extent[0],
                        height: extent[1],
                    },
                })
                .clear_values(std::slice::from_ref(&clear_value));

            let barrier = ImageMemoryBarrier::default()
                .src_access_mask(AccessFlags::empty())
                .dst_access_mask(
                    AccessFlags::COLOR_ATTACHMENT_READ | AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .old_layout(ImageLayout::PRESENT_SRC_KHR)
                .new_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
                .image(self.images[image_index].handle())
                .subresource_range(ImageSubresourceRange {
                    aspect_mask: ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            self.ash_device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::BOTTOM_OF_PIPE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier),
            );

            self.ash_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin,
                SubpassContents::INLINE,
            );

            self.renderer.cmd_draw(command_buffer, reconciled).unwrap();

            self.ash_device.cmd_end_render_pass(command_buffer);
            self.ash_device.end_command_buffer(command_buffer).unwrap();
        }

        let submit_info =
            SubmitInfo::default().command_buffers(std::slice::from_ref(&command_buffer));

        unsafe {
            self.ash_device
                .queue_submit(
                    self.queue.handle(),
                    std::slice::from_ref(&submit_info),
                    fence,
                )
                .unwrap();
        }
    }
}

impl Drop for Debug {
    fn drop(&mut self) {
        unsafe {
            self.ash_device.device_wait_idle().unwrap();
        }

        if let Err(e) = self.renderer.shutdown(&mut self.context) {
            eprintln!("imgui renderer shutdown failed: {e}");
        }
        if let Err(e) = self.platform.shutdown(&mut self.context) {
            eprintln!("imgui platform shutdown failed: {e}");
        }

        unsafe {
            for f in &self.fences {
                self.ash_device.destroy_fence(*f, None);
            }
            for fb in &self.framebuffer {
                self.ash_device.destroy_framebuffer(*fb, None);
            }
            for v in &self.image_views {
                self.ash_device.destroy_image_view(*v, None);
            }
            self.ash_device.destroy_render_pass(self.render_pass, None);
            self.ash_device
                .destroy_command_pool(self.command_pool, None);
        }
    }
}
