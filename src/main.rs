pub mod camera;
pub mod common;
pub mod engine;
pub mod entity;
pub mod player;
pub mod world;

use engine::Engine;
use engine::Flags;
use player::Player;
use std::time;

use crate::common::Updateable;

fn main() {
    let mut flags = Flags::new();

    let mut engine = Engine::new(
        "vox engine using rust",
        1920,
        1080,
        time::Instant::now(),
        flags,
    );

    let mut player = Player::new(
        45.0,
        0.1,
        1000.0,
        100.0,
        100.0,
        10,
        glam::vec3(0.0, 5.0, 0.0),
        glam::Vec3::ONE,
    );

    engine.init_rendering(
        player.get_controller().get_position(),
        player.get_camera().get_fov(),
        player.get_camera().get_near(),
        player.get_camera().get_far(),
    );

    while !engine.get_flags().get_quit_state() {
        engine.event_handling();

        player.collect_inputs(
            engine.get_event(),
            engine.get_x_offset(),
            engine.get_y_offset(),
        );
        player.update(engine.get_delta_time());

        engine.swap();
    }
}
