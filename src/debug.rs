use std::sync::Arc;

use ash::vk::{AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, ClearValue, CommandPool, Extent2D, Fence, Framebuffer, ImageLayout, ImageView, Offset2D, PipelineBindPoint, PipelineStageFlags, Rect2D, RenderPass, RenderPassBeginInfo, RenderPassCreateInfo, SUBPASS_EXTERNAL, SampleCountFlags, SubmitInfo, SubpassContents, SubpassDependency, SubpassDescription};
use ash::vk;
use ash::vk::FramebufferCreateInfo;
use ash::vk::ClearColorValue;
use dear_imgui_ash::AshRenderer;
use dear_imgui_rs::Context;
use dear_imgui_sdl3::process_sys_event;
use vulkano::{image::Image, swapchain::Swapchain};
use vulkano::device::Device;
use vulkano::instance::Instance;
use vulkano::device::Queue;
use vulkano::VulkanObject;
use dear_imgui_reflect::ImGuiReflectExt;

use crate::engine::Engine;
use crate::player::Player;


#[allow(unused)]
pub struct Debug {
    device: Arc<Device>,
    queue: Arc<Queue>,
    instance: Arc<Instance>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,

    ash_device: Arc<ash::Device>,
    ash_instance: Arc<ash::Instance>,

    context: Context,
    renderer: AshRenderer,
    render_pass: RenderPass,
    framebuffer: Vec<Framebuffer>,
    image_views: Vec<ImageView>,
    command_pool: CommandPool
}

impl Debug {
    pub fn new(engine: &Engine) -> Self {
        let library = engine.get_library();
        let device = engine.get_device();
        let instance = engine.get_instance();
        let queue = engine.get_queue();
        let swapchain = engine.get_swapchain();
        let images = engine.get_images();

        let ash_device = Arc::new(unsafe {
            ash::Device::load(&instance.fns().v1_0, device.handle())
        });

        let ash_instance = Arc::new( unsafe {
            ash::Instance::load_with(
                |name| std::mem::transmute(library.get_instance_proc_addr(instance.handle(), name.as_ptr())),
                instance.handle(),
            )
        });

        let mut context = Context::create();
        dear_imgui_sdl3::init_for_vulkan(&mut context, engine.get_window()).unwrap();

        let command_pool = unsafe {
            ash_device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(queue.queue_family_index())
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None
            ).unwrap()
        };

        let attachment = AttachmentDescription::default()
            .format(swapchain.image_format().into())
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
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_READ | AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass = unsafe {
            ash_device.create_render_pass(
                &RenderPassCreateInfo::default()
                    .attachments(std::slice::from_ref(&attachment))
                    .subpasses(std::slice::from_ref(&subpass))
                    .dependencies(std::slice::from_ref(&dependency)),
                None
            )
            .unwrap()
        };

        let extent = swapchain.image_extent();

        let image_views: Vec<ash::vk::ImageView> = images
            .iter()
            .map(|image| unsafe {
                ash_device.create_image_view(
                    &ash::vk::ImageViewCreateInfo::default()
                        .image(image.handle())
                        .view_type(ash::vk::ImageViewType::TYPE_2D)
                        .format(ash::vk::Format::from_raw(swapchain.image_format() as i32))
                        .components(ash::vk::ComponentMapping::default())
                        .subresource_range(ash::vk::ImageSubresourceRange {
                            aspect_mask: ash::vk::ImageAspectFlags::COLOR,
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
                ash_device.create_framebuffer(
                    &FramebufferCreateInfo::default()
                        .render_pass(render_pass)
                        .attachments(std::slice::from_ref(&view))
                        .width(extent[0])
                        .height(extent[1])
                        .layers(1),
                    None

                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        let renderer = AshRenderer::with_default_allocator(
            &ash_instance,
            device.physical_device().handle(),
            (*ash_device).clone(),
            queue.handle(),
            command_pool,
            render_pass,
            &mut context,
            None,
        )
        .unwrap();

        renderer.configure_imgui_context(&mut context);

        Debug { device, 
                queue, 
                instance, 
                swapchain, 
                images,

                ash_device: ash_device.clone(),
                ash_instance: ash_instance.clone(),

                context, 
                renderer, 
                render_pass, 
                framebuffer,
                image_views, 
                command_pool 
            }



    }

    pub fn render(&mut self, engine: &mut Engine, player: &mut Player) {
        let image_index = engine.get_current_image_index();

        for event in engine.get_collected_events() {
            process_sys_event(&event.to_ll().unwrap_or_default());
        }

        dear_imgui_sdl3::sdl3_new_frame(&mut self.context);
        let ui = self.context.frame();

        ui.window("Debug")
        .size([500.0, 500.0], dear_imgui_rs::Condition::FirstUseEver)
        .build(|| {
            ui.text(engine.get_hardware_info());
            ui.text(engine.get_frame_rate().to_string());
            ui.input_reflect("Flags", engine.get_flags_mut());
            ui.input_reflect("Render Mode", engine.get_render_mode_mut());
            ui.input_reflect("Player", player);
    
        });

        let draw_data = self.context.render();

        let command_buffer = unsafe {
            let alloc_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(self.command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            self.ash_device.allocate_command_buffers(&alloc_info).unwrap()[0]
        };

        unsafe {
            self.ash_device.begin_command_buffer(
                command_buffer,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            )
            .unwrap();

            let clear_value = ClearValue {
                color: ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0]},
            };

            let render_pass_begin = RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffer[image_index as usize])
                .render_area(
                    Rect2D {
                        offset: Offset2D {x: 0, y: 0},
                        extent: Extent2D {
                            width: self.swapchain.image_extent()[0],
                            height: self.swapchain.image_extent()[1]
                        }
                    }
                )
                .clear_values(std::slice::from_ref(&clear_value));

            let barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(AccessFlags::empty())
                .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_READ | AccessFlags::COLOR_ATTACHMENT_WRITE)
                .old_layout(ImageLayout::PRESENT_SRC_KHR)
                .new_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.images[image_index as usize].handle())
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            self.ash_device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::BOTTOM_OF_PIPE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier),
            );

            self.ash_device.cmd_begin_render_pass(command_buffer, &render_pass_begin, SubpassContents::INLINE);

            self.renderer.cmd_draw(command_buffer, &draw_data).unwrap();

            self.ash_device.cmd_end_render_pass(command_buffer);
            self.ash_device.end_command_buffer(command_buffer).unwrap();
        }


        let submit_info = SubmitInfo::default()
            .command_buffers(std::slice::from_ref(&command_buffer));

        unsafe {
            self.ash_device.queue_submit(
                self.queue.handle(),
                std::slice::from_ref(&submit_info),
                Fence::null()
            ).unwrap();
        }
    }
}