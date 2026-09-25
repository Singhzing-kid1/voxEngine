use crate::{
    common::{self, HasEntity, Updateable}, debug::Debug, engine::{Engine, Frame, DescriptorResource, Record}, physics::Physics, player::Player, scene::{Scene, Transition}, scenes::MainMenuScene, world::World,
};

use glam::{vec3};

use std::sync::Arc;

use sdl3::{event::Event, keyboard::Keycode};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage}, command_buffer::{
        AutoCommandBufferBuilder, ClearColorImageInfo, CommandBufferUsage,
        CopyBufferToImageInfo, CopyImageInfo,
        PrimaryCommandBufferAbstract,
    }, descriptor_set::DescriptorSet, format::{ClearColorValue, Format}, image::{ImageType, ImageUsage}, memory::allocator::{
        AllocationCreateInfo, MemoryTypeFilter,
    }, pipeline::{
        ComputePipeline, Pipeline, PipelineBindPoint,
    },
};

use crate::{shader::render_compute_shader, shader::resample_compute_shader};

use dear_imgui_reflect::{ImGuiReflect, ImGuiReflectExt};



#[derive(ImGuiReflect)]
pub struct GameplayScene {
    #[imgui(skip)]
    world: World,
    physics: Physics,
    player: Player,
    accumulator: f32,
    paused: bool,

    ray_length: f32,
    max_fog_height: f32, // consider moving to the world and grabbing using a getter
    #[imgui(skip)]
    voxel_set: Option<Arc<DescriptorSet>>,
    #[imgui(skip)]
    render_set: Arc<DescriptorSet>,

    #[imgui(skip)]
    render_pipeline: Arc<ComputePipeline>,
    #[imgui(skip)]
    resample_pipeline: Arc<ComputePipeline>,
}

impl GameplayScene {
    pub fn new(engine: &mut Engine) -> Self {
        let render_shader= render_compute_shader::load(engine.get_device()).expect("cannot load shader");
        let resample_shader= resample_compute_shader::load(engine.get_device()).expect("cannot load shader");

        let render_pipeline = Engine::create_pipeline(render_shader, "main", engine.get_device());
        let resample_pipeline = Engine::create_pipeline(resample_shader, "main", engine.get_device());

        let set_entries = vec![(0, DescriptorResource::ImageView(engine.get_view()))];

        let render_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(render_pipeline.clone(), 0), 
            &set_entries
        );

        let world = World::new(12999003378434, vec3(2000.0, 1000.0, 2000.0), 32);
        let mut physics = Physics::new(&world, vec3(0.0, -9.81, 0.0), 32);

        let (w, h) = engine.get_dimensions();

        let player = Player::new(
            90.0,
            0.1,
            1000.0,
            45.0,
            200.0,
            10,
            w,
            h,
            glam::vec3(100.0, 550.0, 100.0),
            vec3(1.0, 2.0, 1.0),
            0.2,
            &mut physics,
        );

        GameplayScene {
            world, 
            physics,
            player, 
            accumulator: 0.0, 
            paused: false, 
            
            ray_length: 400.0, 
            max_fog_height: 0.0, 
            voxel_set: None,
            render_set,

            render_pipeline,
            resample_pipeline
        }
    }
}

impl Scene for GameplayScene {
    fn on_enter(&mut self, engine: &mut Engine) {
        self.init(engine);
        engine.toggle_mouse(true);
        engine.get_flags_mut().set_capture_mouse_state(true);
    }

    fn on_exit(&mut self, engine: &mut Engine) {
        engine.toggle_mouse(false);
        engine.get_flags_mut().set_capture_mouse_state(false);
    }

    fn on_unpause(&mut self, engine: &mut Engine) {
        engine.toggle_mouse(true);
        engine.get_flags_mut().set_capture_mouse_state(true);
    }

    fn handle_input(&mut self, engine: &mut Engine) {
        if !self.paused {
            self.player.collect_inputs(engine.get_event(), engine.get_x_offset(), engine.get_y_offset());
        }
    }

    fn update(&mut self, dt: f32, engine: &mut Engine) -> Transition {
        for event in engine.get_collected_events() {
            if matches!(event, Event::KeyDown {keycode: Some(Keycode::Escape), .. }) {
                if self.paused {
                    self.paused = !self.paused;
                    return Transition::Unpause;
                } else {
                    self.paused = !self.paused;
                    return Transition::Pause(
                        Box::new(
                            MainMenuScene::new()
                        )
                    );
                }
            }

            if matches!(event, Event::KeyDown {keycode: Some(Keycode::Q), .. }) {
                return Transition::Switch(
                    Box::new(
                        MainMenuScene::new()
                    )
                );
            }
        }

        if !self.paused {
            self.physics.update_loaded_chunks(&self.world, self.player.entity().get_position());

            self.accumulator += dt;
            while self.accumulator > common::TICK_RATE {
                self.physics.step();
                self.player.fixed_update(&mut self.physics);
                self.accumulator -= common::TICK_RATE;
            }

            let alpha = self.accumulator / common::TICK_RATE;
            self.player.update(alpha, &mut self.physics);
        }
        
        Transition::None
    }

    fn render(&mut self, engine: &mut Engine, frame: &mut Frame) {
        self.record(frame, engine);
    }

    fn render_debug(&mut self, engine: &mut Engine, debug: &mut Debug) {
        debug.render(engine, self);
    }

    fn imgui_reflect_dyn(&mut self, ui: &dear_imgui_rs::Ui) -> bool {
        ui.input_reflect("Gameplay Scene", self)
    }
}

impl Record for GameplayScene {
    fn record(&mut self, frame: &mut Frame, engine: &mut Engine) {
        let (resample_image, resample_view) = Engine::create_image(
            ImageType::Dim2d,
            [*engine.get_width() as u32, *engine.get_height() as u32, 1],
            *engine.get_image_format(),
            ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator()
        );

        
        let resample_set_entries = vec![
            (0, DescriptorResource::ImageView(engine.get_view())),
            (1, DescriptorResource::ImageView(resample_view)),
        ];

        let resample_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(self.resample_pipeline.clone(), 0),
            &resample_set_entries,
        );

        let push_data = render_compute_shader::PushConstants {
            pixelToRay: self.player.get_camera().get_pixel_to_ray_matrix(*engine.get_render_scale()).to_cols_array_2d(),
            voxel_resolution: self.world.get_dimensions_as_arr(),
            render_mode: *engine.get_current_render_mode_mut() as u32,
            max_ray_length: self.ray_length,
            max_height: self.max_fog_height,
        };

        let swapchain_image = frame.get_swapchain_image().clone();

        let builder = frame.get_builder_mut();

        unsafe {
            builder
                .clear_color_image(ClearColorImageInfo {
                    clear_value: ClearColorValue::Float([0.0, 0.0, 0.0, 1.0]),
                    ..ClearColorImageInfo::image(engine.get_image())
                })
                .unwrap()
                .bind_pipeline_compute(self.render_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.render_pipeline.layout().clone(),
                    0,
                    vec![
                        self.render_set.clone(),
                        self.voxel_set.as_ref().unwrap().clone(),
                    ],
                )
                .unwrap()
                .push_constants(self.render_pipeline.layout().clone(), 0, push_data)
                .unwrap()
                .dispatch([
                    (*engine.get_width() / *engine.get_render_scale()) as u32 / 8,
                    (*engine.get_height() / *engine.get_render_scale()) as u32 / 8,
                    1,
                ])
                .unwrap()
                .bind_pipeline_compute(self.resample_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.resample_pipeline.layout().clone(),
                    0,
                    vec![resample_set.clone()],
                )
                .unwrap()
                .dispatch([*engine.get_width() as u32 / 8, *engine.get_height() as u32 / 8, 1])
                .unwrap()
                .copy_image(CopyImageInfo::images(
                    resample_image.clone(),
                    swapchain_image,
                ))
                .unwrap();
        };
    }

    fn init(&mut self, engine: &mut Engine) {
        let resolution = self.world.get_dimensions_as_arr();
        let max_height = self.world.get_dimensions().y;

        let (voxels, voxels_view) = Engine::create_image(
            ImageType::Dim3d,
            [resolution[0] / 4, resolution[1] / 4, resolution[2] / 8],
            Format::R32G32B32A32_UINT,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator()
        );

        let (biomes, biomes_view) = Engine::create_image(
            ImageType::Dim2d,
            [resolution[0], resolution[1], 1],
            Format::R8G8B8A8_UNORM,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator()
        );

        let voxel_staging_buffer = Buffer::from_iter(
            engine.get_memory_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            self.world.get_world_as_u32(),
        )
        .unwrap();

        let biomes_staging_buffer = Buffer::from_iter(
            engine.get_memory_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            self.world.get_biomes(),
        )
        .unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            engine.get_command_buffer_allocator(),
            engine.get_queue().queue_family_index(),
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
            .execute(engine.get_queue())
            .unwrap();

        let voxel_set_entries = vec![
            (0, DescriptorResource::ImageView(voxels_view)),
            (1, DescriptorResource::ImageView(biomes_view)),
        ];

        let voxel_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(self.render_pipeline.clone(), 1),
            &voxel_set_entries,
        );

        self.voxel_set = Some(voxel_set);
        self.max_fog_height = max_height * 0.33;

    }
}