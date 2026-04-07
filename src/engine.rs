use std::sync::Arc;
use std::time;

use sdl3::EventPump;
use sdl3::VideoSubsystem;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::video::Window;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::{Buffer, BufferCreateInfo};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::ClearColorImageInfo;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::CopyBufferToImageInfo;
use vulkano::command_buffer::CopyImageInfo;
use vulkano::command_buffer::PrimaryCommandBufferAbstract;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::DescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
    physical::PhysicalDeviceType,
};
use vulkano::format::ClearColorValue;
use vulkano::format::Format;
use vulkano::image::Image;
use vulkano::image::ImageCreateInfo;
use vulkano::image::ImageUsage;
use vulkano::image::view::ImageView;
use vulkano::image::view::ImageViewCreateInfo;
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::PipelineLayout;
use vulkano::pipeline::PipelineShaderStageCreateInfo;
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo};
use vulkano::swapchain::{SurfaceApi, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{VulkanLibrary, VulkanObject};

use crate::cs;
use crate::cs::PushConstants;
use crate::rs;

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
enum RENDERMODE {
    COORD,
    STEPS,
    NORMAL,
    UV,
    DEPTH
}

pub struct Flags {
    quit: bool,
    gravity: bool,
    capture_mouse: bool,
}

impl Flags {
    pub fn new() -> Self {
        Flags {
            quit: false,
            gravity: false,
            capture_mouse: true,
        }
    }

    pub fn get_quit_state(&self) -> bool {
        self.quit
    }

    pub fn get_gravity_state(&self) -> bool {
        self.gravity
    }

    pub fn get_capture_mouse_state(&self) -> bool {
        self.capture_mouse
    }

    pub fn set_quit_state(&mut self, state: bool) {
        self.quit = state;
    }

    pub fn set_gravity_state(&mut self, state: bool) {
        self.gravity = state;
    }

    pub fn set_capture_mouse_state(&mut self, state: bool) {
        self.capture_mouse = state;
    }
}

#[allow(unused)]
pub struct Engine {
    delta_time: u128,
    last_frame: u128,
    start: time::Instant,

    width: u16,
    height: u16,

    sdl_context: sdl3::Sdl,
    video: VideoSubsystem,

    window: Window,

    library: Arc<VulkanLibrary>,
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    render_compute_pipeline: Arc<ComputePipeline>,
    resample_compute_pipeline: Arc<ComputePipeline>,

    voxel_set: Option<Arc<DescriptorSet>>,
    render_set: Arc<DescriptorSet>,

    image: Arc<Image>,
    view: Arc<ImageView>,

    previous_future: Option<Box<dyn GpuFuture + Send + Sync>>,

    event: EventPump,

    x_offset: f32,
    y_offset: f32,
    accum_x: f32,
    accum_y: f32,
    mouse_x: f32,
    mouse_y: f32,
    last_x: f32,
    last_y: f32,

    current_render_mode: RENDERMODE,

    scale: f32,

    x: f32,
    y: f32,

    flags: Flags,
}

impl Engine {
    #[allow(unused_mut)]
    pub fn new(title: &str, width: u16, height: u16, start: time::Instant, flags: Flags) -> Self {
        let sdl_context = sdl3::init().unwrap();
        let video = sdl_context.video().unwrap();
        let event = sdl_context.event_pump().unwrap();

        let window = video
            .window(title, width.into(), height.into())
            .position_centered()
            .vulkan()
            .build()
            .unwrap();

        let extensions = window.vulkan_instance_extensions().unwrap();

        let mut enabled_extensions = vulkano::instance::InstanceExtensions::empty();

        for ext in &extensions {
                match ext.as_str() {
                    "VK_KHR_wayland_surface" => enabled_extensions.khr_wayland_surface = true,
                    "VK_KHR_xlib_surface" => enabled_extensions.khr_xlib_surface = true,
                    "VK_KHR_xcb_surface" => enabled_extensions.khr_xcb_surface = true,
                    "VK_KHR_surface" => enabled_extensions.khr_surface = true,
                    "VK_KHR_win32_surface" => enabled_extensions.khr_win32_surface = true,
                    _ => {
                        eprintln!("Unknown Vulkan instance extension: {}", ext);
                }
            }
        }

        let library = VulkanLibrary::new().expect("no vulkan library or dll");

        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions,
                ..Default::default()
            },
        )
        .expect("could not create vulkan instance");

        let mut surface;

        unsafe {
            let sdl3_surface = window.vulkan_create_surface(instance.handle()).unwrap();
            surface =
                Surface::from_handle(instance.clone(), sdl3_surface, SurfaceApi::Headless, None);
        }

        let surface = Arc::new(surface);

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .expect("could not find physical devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .expect("no device available");

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: queue_family_index as u32,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .expect("could not create logical device");

        let queue = queues.next().unwrap();

        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();

        let image_format = physical_device
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0;

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count.max(3),
                image_format,
                image_extent: [width as u32, height as u32],
                image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
                composite_alpha,
                present_mode: swapchain::PresentMode::Immediate,
                ..Default::default()
            },
        )
        .unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let render_shader = cs::load(device.clone()).expect("cannot load shader");
        let cs = render_shader.entry_point("main").unwrap();
        let stage = PipelineShaderStageCreateInfo::new(cs);
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();

        let render_compute_pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .unwrap();

        let resample_shader = rs::load(device.clone()).expect("cannot load shader");
        let rs = resample_shader.entry_point("main").unwrap();
        let stage = PipelineShaderStageCreateInfo::new(rs);
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();

        let resample_compute_pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .unwrap();

        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: vulkano::image::ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [width as u32, height as u32, 1],
                usage: ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();

        let layout = render_compute_pipeline
            .layout()
            .set_layouts()
            .get(0)
            .unwrap();
        let view = ImageView::new_default(image.clone()).unwrap();

        let render_set = DescriptorSet::new(
            descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::image_view(0, view.clone())],
            [],
        )
        .unwrap();

        let previous_future =
            Some(Box::new(vulkano::sync::now(device.clone())) as Box<dyn GpuFuture + Send + Sync>);

        Engine {
            delta_time: 0,
            last_frame: 0,
            start,

            width,
            height,

            sdl_context,
            video,

            window,

            library,
            instance,
            surface,
            device,
            queue,
            swapchain,
            images,

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,

            render_compute_pipeline,
            resample_compute_pipeline,

            render_set,
            voxel_set: None,

            image,
            view,

            previous_future,

            event,

            current_render_mode: RENDERMODE::STEPS,

            x_offset: 0.0,
            y_offset: 0.0,
            accum_x: 0.0,
            accum_y: 0.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            last_x: 0.0,
            last_y: 0.0,

            scale: 2.5,
            x: 0.0,
            y: 0.0,
            flags,
        }
    }

    pub fn event_handling(&mut self) {
        let current_frame = self.start.elapsed().as_millis();
        self.delta_time = current_frame - self.last_frame;
        self.last_frame = current_frame;

        self.x_offset = self.accum_x - self.last_x;
        self.y_offset = self.last_y - self.accum_y;

        self.last_x = self.accum_x;
        self.last_y = self.accum_y;

        for event in self.event.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.flags.set_quit_state(true);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::G),
                    ..
                } => {
                    self.flags.set_gravity_state(true);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => {
                    self.flags.set_capture_mouse_state(!self.flags.get_capture_mouse_state());
                    self.sdl_context
                        .mouse()
                        .set_relative_mouse_mode(&self.window, self.flags.get_capture_mouse_state());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    let render_mode = self.current_render_mode as u32;
                    let new_render_mode = (render_mode + 1) % 5;
                    self.current_render_mode = match new_render_mode {
                        0 => RENDERMODE::COORD,
                        1 => RENDERMODE::STEPS,
                        2 => RENDERMODE::NORMAL,
                        3 => RENDERMODE::UV,
                        4 => RENDERMODE::DEPTH,
                        _ => RENDERMODE::STEPS,
                    }
                }
                Event::MouseMotion {
                    xrel, yrel, x, y, ..
                } => {
                    if self.flags.get_capture_mouse_state() {
                        self.accum_x -= xrel;
                        self.accum_y += yrel;

                        self.mouse_x = x;
                        self.mouse_y = y;

                        let new_mouse_x = self.mouse_x + xrel;
                        let new_mouse_y = self.mouse_y + yrel;

                        if new_mouse_x < self.width as f32 / 2.0
                            || new_mouse_x > self.width as f32 / 2.0
                            || new_mouse_y < self.height as f32 / 2.0
                            || new_mouse_y > self.height as f32 / 2.0
                        {
                            self.sdl_context.mouse().warp_mouse_in_window(
                                &self.window,
                                self.width as f32 / 2.0,
                                self.height as f32 / 2.0,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn toggle_mouse(&self, toggle: bool) {
        self.sdl_context
            .mouse()
            .set_relative_mouse_mode(&self.window, toggle);
    }

    pub fn get_flags(&self) -> &Flags {
        &self.flags
    }

    pub fn get_event(&self) -> &EventPump {
        &self.event
    }

    pub fn get_x_offset(&self) -> f32 {
        self.x_offset
    }

    pub fn get_y_offset(&self) -> f32 {
        self.y_offset
    }

    pub fn get_delta_time(&self) -> u128 {
        self.delta_time
    }

    pub fn get_dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn send_world_data(&mut self, world: Vec<u32>, resolution: u32) {
        let voxels = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: vulkano::image::ImageType::Dim3d,
                format: Format::R32G32B32A32_UINT,
                extent: [resolution / 4, resolution / 4, resolution / 8],
                usage: ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();

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
            world,
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
            .unwrap();

        let _ = builder
            .build()
            .unwrap()
            .execute(self.queue.clone())
            .unwrap();

        let view =
            ImageView::new(voxels.clone(), ImageViewCreateInfo::from_image(&voxels)).unwrap();

        let layout = self
            .render_compute_pipeline
            .layout()
            .set_layouts()
            .get(0)
            .unwrap();

        let voxel_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::image_view(0, view)],
            [],
        )
        .unwrap();

        self.voxel_set = Some(voxel_set);
    }

    pub fn render(&mut self, pixel_to_ray: glam::Mat4, resolution: u32) {
        let (image_index, _, acquire_future) =
            swapchain::acquire_next_image(self.swapchain.clone(), None).unwrap();

        let swapchain_image = &self.images[image_index as usize];

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let layout = self
            .resample_compute_pipeline
            .layout()
            .set_layouts()
            .get(0)
            .unwrap();

        let resample_image = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: vulkano::image::ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [self.width as u32, self.height as u32, 1],
                usage: ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();

        let resample_view = ImageView::new(
            resample_image.clone(),
            ImageViewCreateInfo::from_image(&resample_image),
        )
        .unwrap();

        let resample_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout.clone(),
            [
                WriteDescriptorSet::image_view(0, self.view.clone()),
                WriteDescriptorSet::image_view(1, resample_view),
            ],
            [],
        )
        .unwrap();

        let push_data = PushConstants {
            pixelToRay: pixel_to_ray.to_cols_array_2d(),
            voxel_resolution: resolution,
            render_mode: self.current_render_mode as u32,
        };

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
                .dispatch([self.width as u32 / 8, self.height as u32 / 8, 1])
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
                    swapchain_image.clone(),
                ))
                .unwrap();
        };

        let command_buffer = builder.build().unwrap();

        let previous_future = self
            .previous_future
            .take()
            .unwrap_or_else(|| Box::new(vulkano::sync::now(self.device.clone())));

        let present = previous_future
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
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
