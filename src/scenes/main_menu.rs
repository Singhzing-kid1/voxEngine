use crate::{engine::Engine, scene::{Scene, Transition}, scenes::GameplayScene};
use sdl3::{event::Event, keyboard::Keycode};
use glam::vec3;

pub struct MainMenuScene;

impl MainMenuScene {
    pub fn new() -> Self { MainMenuScene }
}

impl Scene for MainMenuScene {
    fn on_enter(&mut self, engine: &mut Engine) {
        engine.toggle_mouse(false);
    }

    fn handle_input(&mut self, _engine: &mut Engine) {}

    fn update(&mut self, _dt: f32, engine: &mut Engine) -> Transition {
        for event in engine.get_collected_events() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Transition::Quit,
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    return Transition::Switch(
                        Box::new(
                            GameplayScene::new(engine)
                        )
                    );
                }
                _ => {}
            }
        }

        Transition::None
    }

    fn render(&mut self, engine: &mut Engine) {
        let mut frame = engine.start_frame();

        engine.record_clear(&mut frame, vec3(0.05, 0.05, 0.08));
        engine.finish_frame(frame);
    }

    fn render_debug(&mut self, engine: &mut Engine, debug: &mut crate::debug::Debug) {
        debug.render_main_menu(engine);
    }
}

