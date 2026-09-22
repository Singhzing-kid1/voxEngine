use crate::{
    common::{self, HasEntity, Updateable},
    debug::Debug,
    engine::Engine,
    physics::Physics,
    player::Player,
    scene::{Scene, Transition},
    scenes::MainMenuScene,
    world::World,
};

use glam::{vec3};

use sdl3::{event::Event, keyboard::Keycode};

pub struct GameplayScene {
    world: World,
    physics: Physics,
    player: Player,
    accumulator: f32
}

impl GameplayScene {
    pub fn new(engine: &mut Engine) -> Self {
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

        GameplayScene {world, physics, player, accumulator: 0.0}
    }
}

impl Scene for GameplayScene {
    fn on_enter(&mut self, engine: &mut Engine) {
        engine.send_world_data(&self.world);
        engine.toggle_mouse(true);
    }

    fn on_exit(&mut self, engine: &mut Engine) {
        engine.toggle_mouse(false);
        engine.get_flags_mut().set_capture_mouse_state(false);
    }

    fn handle_input(&mut self, engine: &mut Engine) {
        self.player.collect_inputs(engine.get_event(), engine.get_x_offset(), engine.get_y_offset());
    }

    fn update(&mut self, dt: f32, engine: &mut Engine) -> Transition {
        for event in engine.get_collected_events() {
            if matches!(event, Event::KeyDown {keycode: Some(Keycode::Escape), .. }) {
                return Transition::Switch(
                    Box::new(
                        MainMenuScene::new()
                    )
                );
            }
        }

        self.physics.update_loaded_chunks(&self.world, self.player.entity().get_position());

        self.accumulator += dt;
        while self.accumulator > common::TICK_RATE {
            self.physics.step();
            self.player.fixed_update(&mut self.physics);
            self.accumulator -= common::TICK_RATE;
        }

        let alpha = self.accumulator / common::TICK_RATE;
        self.player.update(alpha, &mut self.physics);

        Transition::None
    }

    fn render(&mut self, engine: &mut Engine) {
        let mut frame = engine.start_frame();

        let view = self.player.get_camera().get_pixel_to_ray_matrix(*engine.get_render_scale());

        engine.record_voxel_pass(&mut frame, view, self.world.get_dimensions_as_arr());

        engine.finish_frame(frame);
    }

    fn render_debug(&mut self, engine: &mut Engine, debug: &mut Debug) {
        debug.render_gameplay(engine, &mut self.player);
    }


}