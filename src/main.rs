pub mod camera;
pub mod common;
pub mod engine;
pub mod entity;
pub mod player;
pub mod world;
pub mod debug;
pub mod perlin;
pub mod shader;

use engine::Engine;
use engine::Flags;
use player::Player;
use std::time;
use world::World;
use debug::Debug;
use crate::common::Updateable;
use glam::vec3;

fn main() {
    let mut flags = Flags::new();

    flags.set_capture_mouse_state(true);

    let mut engine = Engine::new(
        "vox engine using rust",
        2560,
        1080,
        time::Instant::now(),
        flags,
    );
    println!("initialized engine");

    let mut debug = Debug::new(&engine);
    println!("initialized debug ui");

    let (w, h) = engine.get_dimensions();

    let mut player = Player::new(
        90.0,
        0.1,
        1000.0,
        100.0,
        1000.0,
        10,
        w,
        h,
        glam::vec3(4.0, 550.0, 9.0),
        glam::Vec3::ONE,
    );

    println!("start world generation");
    let world = World::new(416120398, vec3(2000.0, 1000.0, 2000.0));

    engine.send_world_data(world.get_world_as_u32(), world.get_dimensions_as_arr());
    println!("sent world data to gpu");

    engine.toggle_mouse(engine.get_flags().get_capture_mouse_state());

    while !engine.get_flags().get_quit_state() {
        engine.event_handling();

        player.collect_inputs(
            engine.get_event(),
            engine.get_x_offset(),
            engine.get_y_offset(),
        );

        player.update(engine.get_delta_time());

        engine.render(player.get_camera().get_pixel_to_ray_matrix(), world.get_dimensions_as_arr());
        debug.render(&mut engine, &mut player);
        engine.present();
    }
}
