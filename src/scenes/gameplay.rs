use crate::{
    common::{self, Buildable, Builder, GlobalLight, HasEntity, LocalLight, Updateable},
    debug::Debug,
    engine::{DescriptorResource, Engine, Frame, Record},
    lights::{light::AnimationType, light_manager::LightManager},
    physics::Physics,
    player::Player,
    scene::{Scene, Transition},
    scenes::MainMenuScene,
    shader::{
        apply_light_shader::{self, Parameters},
        light_flood_shader::Params,
        render_compute_shader::PushConstants,
    },
    texture::Atlas,
    world::World,
};

use glam::{Vec4Swizzles, vec3, vec4};

use std::{cell::RefCell, rc::Rc, sync::Arc};

use sdl3::{
    event::Event,
    keyboard::Keycode::{self},
};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, ClearColorImageInfo, CommandBufferUsage, CopyBufferToImageInfo,
        CopyImageInfo, PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract,
    },
    descriptor_set::DescriptorSet,
    format::{ClearColorValue, Format},
    image::{
        Image, ImageType, ImageUsage,
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    padded::Padded,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
};

use crate::{
    shader::light_flood_shader, shader::render_compute_shader, shader::resample_compute_shader,
};

use dear_imgui_reflect::ImGuiReflect;

#[derive(ImGuiReflect)]
pub struct GameplayScene {
    #[imgui(skip)]
    world: World,
    physics: Physics,
    #[imgui(skip)]
    player: Rc<RefCell<Player>>,
    accumulator: f32,
    paused: bool,

    ray_length: f32,
    max_fog_height: f32,
    #[imgui(skip)]
    render_voxel_set: Option<Arc<DescriptorSet>>,
    #[imgui(skip)]
    lighting_voxel_set: Option<Arc<DescriptorSet>>,
    #[imgui(skip)]
    texture_set: Option<Arc<DescriptorSet>>,
    #[imgui(skip)]
    render_set: Arc<DescriptorSet>,
    #[imgui(skip)]
    resample_set: Arc<DescriptorSet>,
    #[imgui(skip)]
    even_lighting_set: Arc<DescriptorSet>,
    #[imgui(skip)]
    odd_lighting_set: Arc<DescriptorSet>,
    #[imgui(skip)]
    lights_set: Arc<DescriptorSet>,
    #[imgui(skip)]
    apply_light_descriptor_set: Arc<DescriptorSet>,
    #[imgui(skip)]
    light_map_sampler_set: Arc<DescriptorSet>,

    lights: LightManager,
    #[imgui(skip)]
    lights_buffer: Subbuffer<[LocalLight]>,

    #[imgui(skip)]
    resample_image: Arc<Image>,
    #[imgui(skip)]
    albedo: Arc<Image>,
    #[imgui(skip)]
    normals: Arc<Image>,
    #[imgui(skip)]
    shadows: Arc<Image>,
    #[imgui(skip)]
    hit_pos: Arc<Image>,
    #[imgui(skip)]
    post_lighting: Arc<Image>,
    #[imgui(skip)]
    intial_light_map: Arc<Image>,

    #[imgui(skip)]
    render_pipeline: Arc<ComputePipeline>,
    #[imgui(skip)]
    resample_pipeline: Arc<ComputePipeline>,
    #[imgui(skip)]
    light_flood_pipeline: Arc<ComputePipeline>,
    #[imgui(skip)]
    apply_light_pipeline: Arc<ComputePipeline>,

    albedo_atlas: Atlas,
    normals_atlas: Atlas,
}

impl GameplayScene {
    pub fn new(engine: &mut Engine) -> Self {
        let render_shader =
            render_compute_shader::load(engine.get_device()).expect("cannot load shader");
        let resample_shader =
            resample_compute_shader::load(engine.get_device()).expect("cannot load shader");
        let light_flood_shader =
            light_flood_shader::load(engine.get_device()).expect("cannot load shader");
        let apply_light_shader =
            apply_light_shader::load(engine.get_device()).expect("cannot load shader");

        let render_pipeline = Engine::create_pipeline(render_shader, "main", engine.get_device());
        let resample_pipeline =
            Engine::create_pipeline(resample_shader, "main", engine.get_device());
        let light_flood_pipeline =
            Engine::create_pipeline(light_flood_shader, "main", engine.get_device());

        let apply_light_pipeline =
            Engine::create_pipeline(apply_light_shader.clone(), "main", engine.get_device());

        let (albedo, albedo_view) = Engine::create_image(
            ImageType::Dim2d,
            [
                (*engine.get_width() / *engine.get_render_scale()) as u32,
                (*engine.get_height() / *engine.get_render_scale()) as u32,
                1,
            ],
            Format::R8G8B8A8_UNORM,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
        );

        let (normals, normals_view) = Engine::create_image(
            ImageType::Dim2d,
            [
                (*engine.get_width() / *engine.get_render_scale()) as u32,
                (*engine.get_height() / *engine.get_render_scale()) as u32,
                1,
            ],
            Format::R16G16B16A16_SFLOAT,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
        );

        let (shadows, shadows_view) = Engine::create_image(
            ImageType::Dim2d,
            [
                (*engine.get_width() / *engine.get_render_scale()) as u32,
                (*engine.get_height() / *engine.get_render_scale()) as u32,
                1,
            ],
            Format::R8_UNORM,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
        );

        let (hit_pos, hit_pos_view) = Engine::create_image(
            ImageType::Dim2d,
            [
                (*engine.get_width() / *engine.get_render_scale()) as u32,
                (*engine.get_height() / *engine.get_render_scale()) as u32,
                1,
            ],
            Format::R32G32B32A32_SFLOAT,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
        );

        let set_entries = vec![
            (0, DescriptorResource::ImageView(albedo_view.clone())),
            (1, DescriptorResource::ImageView(normals_view.clone())),
            (2, DescriptorResource::ImageView(shadows_view.clone())),
            (3, DescriptorResource::ImageView(hit_pos_view.clone())),
        ];

        let render_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(render_pipeline.clone(), 0),
            &set_entries,
        );

        let (post_lighting, post_lighting_view) = Engine::create_image(
            ImageType::Dim2d,
            [
                (*engine.get_width() / *engine.get_render_scale()) as u32,
                (*engine.get_height() / *engine.get_render_scale()) as u32,
                1,
            ],
            Format::R8G8B8A8_UNORM,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
        );

        let (resample_image, resample_view) = Engine::create_image(
            ImageType::Dim2d,
            [*engine.get_width() as u32, *engine.get_height() as u32, 1],
            *engine.get_image_format(),
            ImageUsage::STORAGE | ImageUsage::TRANSFER_SRC,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
        );

        let resample_set_entries = vec![
            (0, DescriptorResource::ImageView(post_lighting_view.clone())),
            (1, DescriptorResource::ImageView(resample_view)),
        ];

        let resample_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(resample_pipeline.clone(), 0),
            &resample_set_entries,
        );

        let apply_light_output_set_entries =
            vec![(0, DescriptorResource::ImageView(post_lighting_view.clone()))];

        let apply_light_descriptor_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(apply_light_pipeline.clone(), 3),
            &apply_light_output_set_entries,
        );

        let light_map_sampler = Sampler::new(
            engine.get_device(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Nearest,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )
        .unwrap();

        let albedo_atlas = Atlas::builder()
            .add_image_from_sprite_sheet("/textures/voxels_programmer_art.png", [16, 16], [1, 23])
            .set_atlas_resolution([16, 16])
            .set_format(Format::R8G8B8A8_UNORM)
            .set_lod_bias(0.0)
            .build(engine)
            .unwrap();

        let normals_atlas = Atlas::builder()
            .add_image("/textures/voxels_normals.png")
            .add_image("/textures/voxels_normals.png")
            .set_atlas_resolution([16, 16])
            .set_format(Format::R8G8B8A8_UNORM)
            .set_lod_bias(0.0)
            .build(engine)
            .unwrap();

        let (lightmap_1, lightmap_1_view) = Engine::create_image(
            ImageType::Dim3d,
            [128; 3],
            Format::R16G16B16A16_SFLOAT,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
        );

        let (_lightmap_2, lightmap_2_view) = Engine::create_image(
            ImageType::Dim3d,
            [128; 3],
            Format::R16G16B16A16_SFLOAT,
            ImageUsage::STORAGE | ImageUsage::SAMPLED,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
        );

        let even_lighting_set_entries = vec![
            (0, DescriptorResource::ImageView(lightmap_1_view.clone())),
            (1, DescriptorResource::ImageView(lightmap_2_view.clone())),
        ];

        let odd_lighting_set_entries = vec![
            (0, DescriptorResource::ImageView(lightmap_2_view.clone())),
            (1, DescriptorResource::ImageView(lightmap_1_view.clone())),
        ];

        let light_map_sampler_set_entries = vec![(
            0,
            DescriptorResource::SampleTex(lightmap_2_view.clone(), light_map_sampler.clone()),
        )];

        let even_lighting_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(light_flood_pipeline.clone(), 1),
            &even_lighting_set_entries,
        );
        let odd_lighting_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(light_flood_pipeline.clone(), 1),
            &odd_lighting_set_entries,
        );

        let light_map_sampler_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(apply_light_pipeline.clone(), 1),
            &light_map_sampler_set_entries,
        );

        let mut lights = LightManager::new(256);

        let lights_buffer = Buffer::new_slice::<LocalLight>(
            engine.get_memory_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            256,
        )
        .unwrap();

        let lights_set_entries = vec![(0, DescriptorResource::LightBuffer(lights_buffer.clone()))];

        let lights_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(light_flood_pipeline.clone(), 2),
            &lights_set_entries,
        );

        let world = World::new(12999003378434, vec3(2000.0, 1000.0, 2000.0), 32);
        let mut physics = Physics::new(&world, vec3(0.0, -9.81, 0.0), 32);

        let (w, h) = engine.get_dimensions();

        let player = Rc::new(RefCell::new(Player::new(
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
        )));

        lights.add_light(
            vec3(100.0, 450.0, 100.0),
            vec4(1.0, 0.0, 0.0, 5.0),
            None,
            Some(AnimationType::Follow(player.clone())),
            None,
            [2000, 1000, 2000],
        );

        let count = lights.get_amount_of_active_lights() as usize;
        lights_buffer.write().unwrap()[..count]
            .copy_from_slice(&lights.get_lights_as_array()[..count]);

        GameplayScene {
            world,
            physics,
            player,
            accumulator: 0.0,
            paused: false,

            ray_length: 400.0,
            max_fog_height: 0.0,
            render_voxel_set: None,
            lighting_voxel_set: None,
            texture_set: None,
            render_set,
            resample_set,
            even_lighting_set,
            odd_lighting_set,
            lights_set,
            apply_light_descriptor_set,
            light_map_sampler_set,

            lights,
            lights_buffer,

            resample_image,
            albedo,
            normals,
            shadows,
            hit_pos,
            post_lighting,
            intial_light_map: lightmap_1,

            render_pipeline,
            resample_pipeline,
            light_flood_pipeline,
            apply_light_pipeline,

            albedo_atlas,
            normals_atlas,
        }
    }
}

impl GameplayScene {
    fn clear_images(&mut self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) {
        builder
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Float([0.0, 0.0, 0.0, 1.0]),
                ..ClearColorImageInfo::image(self.albedo.clone())
            })
            .unwrap()
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Float([0.0, 0.0, 0.0, 1.0]),
                ..ClearColorImageInfo::image(self.normals.clone())
            })
            .unwrap()
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Float([0.0, 0.0, 0.0, 1.0]),
                ..ClearColorImageInfo::image(self.shadows.clone())
            })
            .unwrap()
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Float([0.0, 0.0, 0.0, 1.0]),
                ..ClearColorImageInfo::image(self.hit_pos.clone())
            })
            .unwrap()
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Float([0.0, 0.0, 0.0, 1.0]),
                ..ClearColorImageInfo::image(self.post_lighting.clone())
            })
            .unwrap()
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Uint([0, 0, 0, 1]),
                ..ClearColorImageInfo::image(self.intial_light_map.clone())
            })
            .unwrap();
    }

    fn render_pass(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        engine: &mut Engine,
        push_data: PushConstants,
    ) {
        unsafe {
            builder
                .bind_pipeline_compute(self.render_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.render_pipeline.layout().clone(),
                    0,
                    vec![
                        self.render_set.clone(),
                        self.render_voxel_set.as_ref().unwrap().clone(),
                        self.texture_set.as_ref().unwrap().clone(),
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
                .unwrap();
        }
    }

    fn lighting_flood_fill_pass(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        push_data: Params,
    ) {
        unsafe {
            builder
                .bind_pipeline_compute(self.light_flood_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.light_flood_pipeline.layout().clone(),
                    0,
                    vec![
                        self.lighting_voxel_set.as_ref().unwrap().clone(),
                        self.even_lighting_set.clone(),
                        self.lights_set.clone(),
                    ],
                )
                .unwrap()
                .push_constants(self.light_flood_pipeline.layout().clone(), 0, push_data)
                .unwrap()
                .dispatch([32, 32, 16])
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.light_flood_pipeline.layout().clone(),
                    0,
                    vec![
                        self.lighting_voxel_set.as_ref().unwrap().clone(),
                        self.odd_lighting_set.clone(),
                        self.lights_set.clone(),
                    ],
                )
                .unwrap()
                .push_constants(self.light_flood_pipeline.layout().clone(), 0, push_data)
                .unwrap()
                .dispatch([32, 32, 16])
                .unwrap();
        }
    }

    fn apply_light_pass(
        &mut self,
        engine: &mut Engine,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        push_data: Parameters,
    ) {
        unsafe {
            builder
                .bind_pipeline_compute(self.apply_light_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.apply_light_pipeline.layout().clone(),
                    0,
                    vec![
                        self.render_set.clone(),
                        self.light_map_sampler_set.clone(),
                        self.lighting_voxel_set.as_ref().unwrap().clone(),
                        self.apply_light_descriptor_set.clone(),
                    ],
                )
                .unwrap()
                .push_constants(self.apply_light_pipeline.layout().clone(), 0, push_data)
                .unwrap()
                .dispatch([
                    (*engine.get_width() / *engine.get_render_scale()) as u32 / 8,
                    (*engine.get_height() / *engine.get_render_scale()) as u32 / 8,
                    1,
                ])
                .unwrap();
        }
    }

    fn upscale_pass(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        engine: &mut Engine,
    ) {
        unsafe {
            builder
                .bind_pipeline_compute(self.resample_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.resample_pipeline.layout().clone(),
                    0,
                    vec![self.resample_set.clone()],
                )
                .unwrap()
                .dispatch([
                    *engine.get_width() as u32 / 8,
                    *engine.get_height() as u32 / 8,
                    1,
                ])
                .unwrap();
        }
    }

    fn blit_to_swapchain(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        swapchain_image: Arc<Image>,
    ) {
        builder
            .copy_image(CopyImageInfo::images(
                self.resample_image.clone(),
                swapchain_image,
            ))
            .unwrap();
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
            self.player.borrow_mut().collect_inputs(
                engine.get_event(),
                engine.get_x_offset(),
                engine.get_y_offset(),
            );
        }
    }

    fn update(&mut self, dt: f32, engine: &mut Engine) -> Transition {
        for event in engine.get_collected_events() {
            if matches!(
                event,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
            ) {
                if self.paused {
                    self.paused = !self.paused;
                    return Transition::Unpause;
                } else {
                    self.paused = !self.paused;
                    return Transition::Pause(Box::new(MainMenuScene::new()));
                }
            }

            if matches!(
                event,
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                }
            ) {
                return Transition::Switch(Box::new(MainMenuScene::new()));
            }

            if matches!(
                event,
                Event::KeyDown {
                    keycode: Some(Keycode::L),
                    ..
                }
            ) {
                self.lights.add_light(
                    self.player.borrow().entity().get_position(),
                    vec4(1.0, 1.0, 1.0, 5.0),
                    Some(1000.0),
                    None,
                    None,
                    self.world.get_dimensions_as_arr(),
                );

                return Transition::None;
            }
        }

        if !self.paused {
            self.physics
                .update_loaded_chunks(&self.world, self.player.borrow().entity().get_position());

            self.accumulator += dt;
            while self.accumulator > common::TICK_RATE {
                self.physics.step();
                self.player.borrow_mut().fixed_update(&mut self.physics);
                self.lights.update_lights();
                self.accumulator -= common::TICK_RATE;
            }

            let alpha = self.accumulator / common::TICK_RATE;
            self.player.borrow_mut().update(alpha, &mut self.physics);
        }

        Transition::None
    }

    fn render(&mut self, engine: &mut Engine, frame: &mut Frame) {
        self.record(frame, engine);
    }

    fn render_debug(&mut self, engine: &mut Engine, debug: &mut Debug) {
        debug.render(engine, self);
    }

    fn imgui_reflect_dyn(&mut self, ui: &mut dear_imgui_reflect::Inspector<'_, '_>) -> bool {
        let mut changed = ui.input("Gameplay Scene", self);

        ui.ui().with_indent(|| {
            let mut player = self.player.borrow_mut();
            changed |= ui.input("Player", &mut *player);
        });

        changed
    }
}

impl Record for GameplayScene {
    fn record(&mut self, frame: &mut Frame, engine: &mut Engine) {
        let sun = GlobalLight {
            position: vec3(500.0, 5000.0, 500.0),
            color: vec3(1.0, 1.0, 1.0),
            intensity: 0.1,
        };

        let count = self.lights.get_amount_of_active_lights() as usize;
        self.lights_buffer.write().unwrap()[..count]
            .copy_from_slice(&self.lights.get_lights_as_array()[..count]);

        let render_push_data = render_compute_shader::PushConstants {
            pixelToRay: self
                .player
                .borrow()
                .get_camera()
                .get_pixel_to_ray_matrix(*engine.get_render_scale())
                .to_cols_array_2d(),
            voxel_resolution: self.world.get_dimensions_as_arr(),
            render_mode: *engine.get_current_render_mode_mut() as u32,
            max_ray_length: self.ray_length,
            max_height: self.max_fog_height,
            lod_bias: Padded(self.albedo_atlas.get_atlas_lod_bias()),
            sun: render_compute_shader::GlobalLight::from(sun),
        };

        let player_coord = self
            .player
            .borrow()
            .get_camera()
            .get_pixel_to_ray_matrix(*engine.get_render_scale())
            .w_axis
            .xyz();
        let light_volume_min = (player_coord.floor().as_ivec3() - 64).to_array();

        let lighting_push_data = Params {
            lightVolumeMin: light_volume_min,
            localLightCount: self.lights.get_amount_of_active_lights(),
            falloffPerVoxel: 0.85,
        };

        let apply_lighting_push_data = Parameters {
            sun: apply_light_shader::GlobalLight::from(sun),
            lightVolumeMin: Padded(light_volume_min),
            volumeSize: [128.0; 3],
            localStrength: 1.0,
            aoStrength: Padded(1.0),
            skyColor: vec3(0.1, 0.1, 0.1).to_array(),
        };

        let swapchain_image = frame.get_swapchain_image().clone();

        let builder = frame.get_builder_mut();

        self.clear_images(builder);

        self.render_pass(builder, engine, render_push_data);

        self.lighting_flood_fill_pass(builder, lighting_push_data);
        self.lighting_flood_fill_pass(builder, lighting_push_data);
        self.lighting_flood_fill_pass(builder, lighting_push_data);
        self.lighting_flood_fill_pass(builder, lighting_push_data);

        self.lighting_flood_fill_pass(builder, lighting_push_data);
        self.lighting_flood_fill_pass(builder, lighting_push_data);
        self.lighting_flood_fill_pass(builder, lighting_push_data);
        self.lighting_flood_fill_pass(builder, lighting_push_data);

        self.apply_light_pass(engine, builder, apply_lighting_push_data);

        self.upscale_pass(builder, engine);

        self.blit_to_swapchain(builder, swapchain_image);
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
            engine.get_memory_allocator(),
        );

        let (biomes, biomes_view) = Engine::create_image(
            ImageType::Dim2d,
            [resolution[0], resolution[2], 1],
            Format::R8G8B8A8_UNORM,
            ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            MemoryTypeFilter::PREFER_DEVICE,
            engine.get_memory_allocator(),
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
            (0, DescriptorResource::ImageView(voxels_view.clone())),
            (1, DescriptorResource::ImageView(biomes_view)),
        ];

        let render_voxel_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(self.render_pipeline.clone(), 1),
            &voxel_set_entries,
        );

        let texture_set_entries = vec![
            (
                0,
                DescriptorResource::Sample(self.albedo_atlas.get_atlas_sampler()),
            ),
            (
                1,
                DescriptorResource::ImageView(self.albedo_atlas.get_atlas()),
            ),
            (
                2,
                DescriptorResource::Sample(self.normals_atlas.get_atlas_sampler()),
            ),
            (
                3,
                DescriptorResource::ImageView(self.normals_atlas.get_atlas()),
            ),
        ];

        let texture_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(self.render_pipeline.clone(), 2),
            &texture_set_entries,
        );

        let lighting_voxel_set_entries =
            vec![(0, DescriptorResource::ImageView(voxels_view.clone()))];

        let lighting_voxel_set = Engine::create_descriptor_set(
            engine.get_descriptor_set_allocator(),
            Engine::get_layout(self.light_flood_pipeline.clone(), 0),
            &lighting_voxel_set_entries,
        );

        self.render_voxel_set = Some(render_voxel_set);
        self.lighting_voxel_set = Some(lighting_voxel_set);
        self.texture_set = Some(texture_set);
        self.max_fog_height = max_height * 0.50;
    }
}
