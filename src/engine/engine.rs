use std::{sync::Arc, time::Instant};

use vulkano::{
    VulkanLibrary, VulkanObject,
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    format::Format,
    image::{Image, ImageType, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator},
    swapchain::{
        PresentMode, Surface, SurfaceApi, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
    },
    sync::{GpuFuture, now, semaphore::Semaphore},
};

use sdl3::{EventPump, Sdl, VideoSubsystem, event::Event, video::Window};

use getset::{CloneGetters, CopyGetters, Getters, MutGetters};

use super::{Flags, helpers};

#[derive(CopyGetters, CloneGetters, Getters, MutGetters)]
pub struct Engine {
    #[getset(get_clone = "pub with_prefix")]
    pub(super) library: Arc<VulkanLibrary>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) instance: Arc<Instance>,

    pub(super) surface: Arc<Surface>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) device: Arc<Device>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) queue: Arc<Queue>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) swapchain: Arc<Swapchain>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) images: Vec<Arc<Image>>,

    #[getset(get_clone = "pub with_prefix")]
    pub(super) memory_allocator: Arc<StandardMemoryAllocator>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    #[getset(get = "pub with_prefix")]
    pub(super) image_format: Format,

    #[getset(get_clone = "pub with_prefix")]
    pub(super) image: Arc<Image>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) view: Arc<ImageView>,

    pub(super) previous_future: Option<Box<dyn GpuFuture + Send + Sync>>,
    #[getset(get_copy = "pub with_prefix")]
    pub(super) current_image_index: u32,
    pub(super) current_acquire_future: Option<SwapchainAcquireFuture>,

    #[getset(get_clone = "pub with_prefix")]
    pub(super) render_complete_semaphore: Arc<Semaphore>,

    #[getset(get = "pub with_prefix")]
    pub(super) event: EventPump,

    #[getset(get = "pub with_prefix")]
    pub(super) collected_events: Vec<Event>,

    #[getset(get_copy = "pub with_prefix")]
    pub(super) delta_time: f64,
    pub(super) last_frame: Instant,
    pub(super) start: Instant,

    #[getset(get = "pub with_prefix")]
    pub(super) render_scale: u16,

    #[getset(get = "pub with_prefix")]
    pub(super) width: u16,
    #[getset(get = "pub with_prefix")]
    pub(super) height: u16,

    #[getset(get_copy = "pub with_prefix")]
    pub(super) x_offset: f32,
    #[getset(get_copy = "pub with_prefix")]
    pub(super) y_offset: f32,

    pub(super) accum_x: f32,
    pub(super) accum_y: f32,

    pub(super) mouse_x: f32,
    pub(super) mouse_y: f32,

    pub(super) last_x: f32,
    pub(super) last_y: f32,

    pub(super) scale: f32,

    pub(super) x: f32,
    pub(super) y: f32,

    #[getset(get_copy = "pub with_prefix", get_mut = "pub with_prefix")]
    pub(super) flags: Flags,

    pub(super) context: Sdl,
    pub(super) video: VideoSubsystem,

    #[getset(get = "pub with_prefix")]
    pub(super) window: Window,
}

impl Engine {
    pub fn new(title: &str, start: Instant, render_scale: u16, flags: Flags) -> Self {
        let context = sdl3::init().expect("could not initialize SDL3");
        let video = context.video().expect("could not create video subsystem");
        let event = context.event_pump().expect("could not get event pump");

        let display = video
            .get_primary_display()
            .expect("could not get primary display");

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
            .expect("could not build window");

        let extensions = window
            .vulkan_instance_extensions()
            .expect("could not get vulkan instance extensions from SDL3 window");

        let mut enabled_extensions = InstanceExtensions::empty();

        for ext in &extensions {
            match ext.as_str() {
                "VK_KHR_wayland_surface" => enabled_extensions.khr_wayland_surface = true,
                "VK_KHR_xlib_surface" => enabled_extensions.khr_xlib_surface = true,
                "VK_KHR_xcb_surface" => enabled_extensions.khr_xcb_surface = true,
                "VK_KHR_win32_surface" => enabled_extensions.khr_win32_surface = true,
                "VK_KHR_surface" => enabled_extensions.khr_surface = true,
                _ => {
                    eprintln!("Unknown Vulkan instance extension: {}", ext)
                }
            }
        }

        let library = VulkanLibrary::new().expect("no vulkan library or dll available");

        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions,
                ..Default::default()
            },
        )
        .expect("could not create vulkan instance");

        let surface;

        unsafe {
            let sdl3_surface = window
                .vulkan_create_surface(instance.handle())
                .expect("could not create vulkan surface from SDL3 window");
            surface =
                Surface::from_handle(instance.clone(), sdl3_surface, SurfaceApi::Headless, None);
        }

        let surface = Arc::new(surface);

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let device_features = DeviceFeatures {
            sampler_anisotropy: true,
            dynamic_rendering: true,
            ..Default::default()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .expect("could not find physical devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter(|p| p.supported_features().contains(&device_features))
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
                enabled_features: device_features,
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
                image_usage: ImageUsage::COLOR_ATTACHMENT
                    | ImageUsage::TRANSFER_DST
                    | ImageUsage::STORAGE,
                composite_alpha,
                present_mode: PresentMode::Immediate,
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

        let previous_future =
            Some(Box::new(now(device.clone())) as Box<dyn GpuFuture + Send + Sync>);

        let render_complete_semaphore =
            Arc::new(Semaphore::new(device.clone(), Default::default()).unwrap());

        let (image, view) = helpers::create_image(
            ImageType::Dim2d,
            [
                (width / render_scale) as u32,
                (height / render_scale) as u32,
                1,
            ],
            image_format,
            ImageUsage::STORAGE
                | ImageUsage::TRANSFER_SRC
                | ImageUsage::TRANSFER_DST
                | ImageUsage::COLOR_ATTACHMENT,
            MemoryTypeFilter::PREFER_DEVICE,
            memory_allocator.clone(),
        );

        Self {
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

            image_format,

            image,
            view,

            previous_future,
            current_image_index: 0,
            current_acquire_future: None,

            render_complete_semaphore,

            event,
            collected_events: Vec::new(),

            delta_time: 0f64,
            last_frame: start,
            start,

            render_scale,

            width,
            height,

            x_offset: 0f32,
            y_offset: 0f32,

            accum_x: 0f32,
            accum_y: 0f32,

            mouse_x: 0f32,
            mouse_y: 0f32,

            last_x: 0f32,
            last_y: 0f32,

            scale: 1f32,

            x: 0f32,
            y: 0f32,

            flags,

            context,
            video,

            window,
        }
    }

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
}
