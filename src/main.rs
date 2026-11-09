pub mod common;
pub mod engine;
pub mod lights;
pub mod physics;
pub mod player;
pub mod scenes;
pub mod shader;
pub mod texture;
pub mod world;

use engine::{Debug, Engine, Flags, scene::SceneManager};
use scenes::MainMenuScene;
use std::time;

fn main() {
    let mut flags = Flags::default();

    flags.set_capture_mouse(false);

    let mut engine = Engine::new("vox engine using rust", time::Instant::now(), 3, flags);

    let mut debug = Debug::new(&engine);

    let mut manager = SceneManager::new(Box::new(MainMenuScene::new()), &mut engine);

    while !engine.get_flags().get_quit() {
        engine.frame_start();
        engine.event_handling();

        let dt_ms = engine.get_delta_time();

        let mut frame_time = dt_ms as f32 / 1000.0;
        if frame_time > common::MAX_FRAME_TIME {
            frame_time = common::MAX_FRAME_TIME;
        }

        manager.handle_input(&mut engine);

        if !manager.update(frame_time, &mut engine) {
            engine.get_flags_mut().set_quit(true);
        }

        manager.render(&mut engine);
        manager.render_debug(&mut engine, &mut debug);

        engine.present();
        engine.frame_end(500);
    }
}
