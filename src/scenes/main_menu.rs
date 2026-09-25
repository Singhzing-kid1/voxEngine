use crate::{engine::{Engine, Frame}, scene::{Scene, Transition}, scenes::GameplayScene};
use sdl3::{event::Event, keyboard::Keycode};
use glam::vec4;

use dear_imgui_reflect::{ImGuiReflect, ImGuiReflectExt};

#[derive(ImGuiReflect)]
pub struct MainMenuScene;

impl MainMenuScene {
    pub fn new() -> Self { MainMenuScene }
}

impl Scene for MainMenuScene {
    fn on_enter(&mut self, engine: &mut Engine) {
        engine.toggle_mouse(false);
        engine.get_flags_mut().set_capture_mouse_state(false);
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

    fn render(&mut self, engine: &mut Engine, frame: &mut Frame) {
        engine.record_clear(frame, vec4(0.05, 0.05, 0.08, 1.0));
    }

    fn render_as_overlay(&mut self, engine: &mut Engine, frame: &mut Frame) {
        engine.record_blur(frame);
    }

    fn render_debug(&mut self, engine: &mut Engine, debug: &mut crate::debug::Debug) {
        debug.render(engine, self);
    }

    fn imgui_reflect_dyn(&mut self, ui: &dear_imgui_rs::Ui) -> bool {
        ui.input_reflect("Main Menu Scene", self)
    }
}

