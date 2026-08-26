pub mod camera;
pub mod common;
pub mod debug;
pub mod engine;
pub mod entity;
pub mod perlin;
pub mod physics;
pub mod player;
pub mod shader;
pub mod world;

use crate::common::HasEntity;
use crate::common::Updateable;
use debug::Debug;
use engine::Engine;
use engine::Flags;
use glam::vec3;
use physics::Physics;
use player::Player;
use std::time;
use world::World;

fn main() {
    let mut flags = Flags::new();

    flags.set_capture_mouse_state(true);

    let mut engine = Engine::new(
        "vox engine using rust",
        time::Instant::now(),
        flags,
    );
    println!("initialized engine");

    let mut debug = Debug::new(&engine);
    println!("initialized debug ui");

    let (w, h) = engine.get_dimensions();

    println!("start world generation");
    let world = World::new(416120398, vec3(2000.0, 1000.0, 2000.0));
    println!("world generation finished");

    println!("creating physics engine");
    let mut physics = Physics::new(&world, vec3(0.0, -9.81, 0.0), 32);

    engine.send_world_data(world.get_world_as_u32(), world.get_dimensions_as_arr());
    println!("sent world data to gpu");

    engine.toggle_mouse(engine.get_flags().get_capture_mouse_state());

    println!("creating player");
    let mut player = Player::new(
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
        &mut physics
    );

    while !engine.get_flags().get_quit_state() {
        engine.frame_start();
        let view = player.get_camera().get_pixel_to_ray_matrix();
        engine.event_handling();

        let dt = engine.get_delta_time();

        if dt > 30 {
            println!("SPIKE: {}ms", dt);
        }

        player.collect_inputs(
            engine.get_event(),
            engine.get_x_offset(),
            engine.get_y_offset(),
        );

        physics.update_loaded_chunks(&world, player.entity().get_position());

        physics.step();

        player.update(engine.get_delta_time(), &mut physics);

        engine.render(view, world.get_dimensions_as_arr());
        debug.render(&mut engine, &mut player);
        engine.present();
        engine.frame_end(60);
    }
}
