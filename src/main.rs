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
pub mod scene;
pub mod scenes;

use debug::Debug;
use engine::Engine;
use engine::Flags;
use std::time;
use scene::SceneManager;
use scenes::{MainMenuScene};

fn main() {
    let mut flags = Flags::new();

    flags.set_capture_mouse_state(false);

    let mut engine = Engine::new("vox engine using rust", time::Instant::now(), 3, flags);

    let mut debug = Debug::new(&engine);

    let mut manager = SceneManager::new(Box::new(MainMenuScene::new()), &mut engine);    

    while !engine.get_flags().get_quit_state() {
        engine.frame_start();
        engine.event_handling();

        let dt_ms = engine.get_delta_time();

        let mut frame_time = dt_ms as f32 / 1000.0;
        if frame_time > common::MAX_FRAME_TIME {
            frame_time = common::MAX_FRAME_TIME;
        }

        manager.handle_input(&mut engine);

        if !manager.update(frame_time, &mut engine) {
            engine.get_flags_mut().set_quit_state(true);
        }

        manager.render(&mut engine);
        manager.render_debug(&mut engine, &mut debug);

        engine.present();
        engine.frame_end(500);
    }
}
