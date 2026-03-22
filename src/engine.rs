use std::sync::Arc;
use std::time;

use sdl3::EventPump;
use sdl3::VideoSubsystem;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::video::Window;

use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
    physical::PhysicalDeviceType,
};
use vulkano::image::Image;
use vulkano::image::ImageUsage;
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo};
use vulkano::swapchain::{SurfaceApi, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{VulkanLibrary, VulkanObject};

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

    event: EventPump,

    x_offset: f32,
    y_offset: f32,
    accum_x: f32,
    accum_y: f32,
    mouse_x: f32,
    mouse_y: f32,
    last_x: f32,
    last_y: f32,

    flags: Flags,
}

impl Engine {
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
                "VK_KHR_surface" => enabled_extensions.khr_surface = true,
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

        let (mut swapchain, images) = Swapchain::new(
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

            event,

            x_offset: 0.0,
            y_offset: 0.0,
            accum_x: 0.0,
            accum_y: 0.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            last_x: 0.0,
            last_y: 0.0,

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
                Event::MouseMotion {
                    xrel, yrel, x, y, ..
                } => {
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
                _ => {}
            }
        }
    }

    pub fn init_rendering(&mut self, position: glam::Vec3, fov: f32, near: f32, far: f32) {
        if self.flags.get_capture_mouse_state() {
            self.sdl_context
                .mouse()
                .set_relative_mouse_mode(&self.window, true);
        }

        let aspect = (self.height / self.width) as f32;
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

    pub fn swap(&self) {
        let (image_index, _, acquire_future) =
            swapchain::acquire_next_image(self.swapchain.clone(), None).unwrap();

        let present = acquire_future
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        present.unwrap();
    }
}
