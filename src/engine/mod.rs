use std::{
    sync::Arc, time::{self},
};

use sdl3::{
    EventPump, VideoSubsystem,
    event::Event,
    video::Window,
};

use vulkano::{
    VulkanLibrary, VulkanObject,
    buffer::Subbuffer,
    command_buffer::{
        AutoCommandBufferBuilder,
        PrimaryAutoCommandBuffer,
        allocator::StandardCommandBufferAllocator,
    },
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::PhysicalDeviceType,
    },
    format::{Format},
    image::{Image, ImageType, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{
        MemoryTypeFilter,
        StandardMemoryAllocator,
    },
    pipeline::{
        ComputePipeline,
    },
    swapchain::{
        self, Surface, SurfaceApi, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
    },
    sync::{GpuFuture, semaphore::Semaphore},
};

use crate::{common::RayHit,
    shader::*
};

use dear_imgui_reflect::ImGuiReflect;
use getset::{CloneGetters, CopyGetters, Getters, MutGetters};


mod helpers;
mod render;
mod raycast;
mod event;
mod record;

#[derive(ImGuiReflect, Copy, Clone)]
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

#[derive(Debug, Clone, Copy, ImGuiReflect)]
#[imgui(enum_style = "dropdown")]
#[allow(unused)]
pub enum RENDERMODE {
    DEFAULT,
    COORD,
    STEPS,
    NORMAL,
    UV,
    DEPTH,
}

#[derive(Clone)]
pub(crate) enum DescriptorResource {
    ImageView(Arc<ImageView>),
    RayBuffer(Subbuffer<RayHit>),
}

impl DescriptorResource {
    fn into_write(self, binding: u32) -> WriteDescriptorSet {
        match self {
            DescriptorResource::ImageView(view) => WriteDescriptorSet::image_view(binding, view),
            DescriptorResource::RayBuffer(buffer) => WriteDescriptorSet::buffer(binding, buffer),
        }
    }
}

#[derive(Getters, MutGetters)]
pub struct Frame {
    #[getset(get_mut = "pub with_prefix")]
    builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    #[getset(get = "pub with_prefix")]
    swapchain_image: Arc<Image>,

    acquire_future: SwapchainAcquireFuture,
}

#[derive(CopyGetters, Getters, MutGetters, CloneGetters)]
#[allow(unused)]
pub struct Engine {
    #[getset(get_copy = "pub with_prefix")]
    delta_time: u128,
    last_frame: time::Instant,
    start: time::Instant,

    width: u16,
    height: u16,

    #[getset(get = "pub with_prefix")]
    render_scale: u16,

    #[getset(get_clone = "pub with_prefix")]
    library: Arc<VulkanLibrary>,
    #[getset(get_clone = "pub with_prefix")]
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    #[getset(get_clone = "pub with_prefix")]
    device: Arc<Device>,
    #[getset(get_clone = "pub with_prefix")]
    queue: Arc<Queue>,
    #[getset(get_clone = "pub with_prefix")]
    swapchain: Arc<Swapchain>,
    #[getset(get_clone = "pub with_prefix")]
    images: Vec<Arc<Image>>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    render_compute_pipeline: Arc<ComputePipeline>,
    resample_compute_pipeline: Arc<ComputePipeline>,

    raycast_compute_pipeline: Arc<ComputePipeline>,

    pause_blur_compute_pipeline: Arc<ComputePipeline>,

    voxel_set: Option<Arc<DescriptorSet>>,
    render_set: Arc<DescriptorSet>,
    image_format: Format,

    image: Arc<Image>,
    view: Arc<ImageView>,

    previous_future: Option<Box<dyn GpuFuture + Send + Sync>>,
    #[getset(get_copy = "pub with_prefix")]
    current_image_index: u32,
    current_acquire_future: Option<SwapchainAcquireFuture>,

    #[getset(get_clone = "pub with_prefix")]
    render_complete_semaphore: Arc<Semaphore>,

    #[getset(get = "pub with_prefix")]
    event: EventPump,

    #[getset(get = "pub with_prefix")]
    collected_events: Vec<Event>,

    #[getset(get_copy = "pub with_prefix")]
    x_offset: f32,
    #[getset(get_copy = "pub with_prefix")]
    y_offset: f32,
    accum_x: f32,
    accum_y: f32,
    mouse_x: f32,
    mouse_y: f32,
    last_x: f32,
    last_y: f32,

    #[getset(get_mut = "pub with_prefix")]
    current_render_mode: RENDERMODE,
    #[getset(get_mut = "pub with_prefix")]
    ray_length: f32,

    scale: f32,

    max_fog_height: f32,

    x: f32,
    y: f32,

    #[getset(get_copy = "pub with_prefix", get_mut = "pub with_prefix")]
    flags: Flags,

    sdl_context: sdl3::Sdl,
    video: VideoSubsystem,

    #[getset(get = "pub with_prefix")]
    window: Window,
}

impl Engine {
    #[allow(unused_mut)]
    pub fn new(title: &str, start: time::Instant, render_scale: u16, flags: Flags) -> Self {
        let sdl_context = sdl3::init().unwrap();
        let video = sdl_context.video().unwrap();
        let event = sdl_context.event_pump().unwrap();

        let display = video.get_primary_display().unwrap();
        let (width, height) = (
            display.get_mode().unwrap().w as u16,
            display.get_mode().unwrap().h as u16,
        );

        let window = video
            .window(title, width.into(), height.into())
            .borderless()
            .position(0, 0)
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
            .unwrap()
            .iter()
            .find(|(f, _)| *f == Format::R8G8B8A8_UNORM)
            .map(|(f, _)| *f)
            .unwrap_or_else(|| {
                physical_device
                    .surface_formats(&surface, Default::default())
                    .unwrap()[0]
                    .0
            });

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count.max(3),
                image_format,
                image_extent: [width as u32, height as u32],
                image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST | ImageUsage::STORAGE,
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

        let render_shader =
            render_compute_shader::load(device.clone()).expect("cannot load shader");

        let resample_shader =
            resample_compute_shader::load(device.clone()).expect("cannot load shader");

        let raycast_shader = raycast_shader::load(device.clone()).expect("cannot load shader");

        let pause_blue_shader = pause_blur_shader::load(device.clone()).expect("cannot load shader");

        let render_compute_pipeline =
            Engine::create_pipeline(render_shader, "main", device.clone());

        let resample_compute_pipeline =
            Engine::create_pipeline(resample_shader, "main", device.clone());

        let raycast_compute_pipeline =
            Engine::create_pipeline(raycast_shader, "main", device.clone());

        let pause_blur_compute_pipeline = Engine::create_pipeline(pause_blue_shader, "main", device.clone());

        let (image, view) = Engine::create_image(
            ImageType::Dim2d,
            [
                (width / render_scale) as u32,
                (height / render_scale) as u32,
                1,
            ],
            image_format,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            memory_allocator.clone(),
        );

        let set_entries = vec![(0, DescriptorResource::ImageView(view.clone()))];

        let render_set = Engine::create_descriptor_set(
            descriptor_set_allocator.clone(),
            Engine::get_layout(render_compute_pipeline.clone(), 0),
            &set_entries,
        );

        let previous_future =
            Some(Box::new(vulkano::sync::now(device.clone())) as Box<dyn GpuFuture + Send + Sync>);

        let render_complete_semaphore = Arc::new(
            vulkano::sync::semaphore::Semaphore::new(device.clone(), Default::default()).unwrap(),
        );

        Engine {
            delta_time: 0,
            last_frame: start,
            start,

            width,
            height,
            render_scale,

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

            raycast_compute_pipeline,

            pause_blur_compute_pipeline,

            render_set,
            voxel_set: None,

            image_format,

            image,
            view,

            previous_future,
            current_image_index: 0,
            current_acquire_future: None,
            render_complete_semaphore,

            event,
            collected_events: Vec::new(),

            current_render_mode: RENDERMODE::DEFAULT,
            ray_length: 450.0,

            x_offset: 0.0,
            y_offset: 0.0,
            accum_x: 0.0,
            accum_y: 0.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            last_x: 0.0,
            last_y: 0.0,

            scale: 2.5,

            max_fog_height: 0.0,

            x: 0.0,
            y: 0.0,
            flags,
        }
    }
}

