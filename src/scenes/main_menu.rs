use crate::{
    engine::{
        Debug, Engine, Frame, Record,
        scene::{Scene, Transition},
    },
    scenes::GameplayScene,
};
use rapier3d::glamx::vec4;
use sdl3::{event::Event, keyboard::Keycode};

use dear_imgui_reflect::ImGuiReflect;
use vulkano::{command_buffer::ClearColorImageInfo, format::ClearColorValue};

#[derive(ImGuiReflect)]
pub struct MainMenuScene;

impl MainMenuScene {
    pub fn new() -> Self {
        MainMenuScene
    }
}

impl Scene for MainMenuScene {
    fn on_enter(&mut self, engine: &mut Engine) {
        engine.toggle_mouse(false);
        engine.get_flags_mut().set_capture_mouse(false);
    }

    fn on_exit(&mut self, _engine: &mut Engine) {}

    fn handle_input(&mut self, _engine: &mut Engine) {}

    fn update(&mut self, _dt: f32, engine: &mut Engine) -> Transition {
        for event in engine.get_collected_events() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Transition::Quit,
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    return Transition::Switch(Box::new(GameplayScene::new(engine)));
                }
                _ => {}
            }
        }

        Transition::None
    }

    fn render(&mut self, engine: &mut Engine, frame: &mut Frame) {
        self.record(frame, engine);
    }

    fn render_debug(&mut self, engine: &mut Engine, debug: &mut Debug) {
        debug.render(engine, self);
    }
}

impl Record for MainMenuScene {
    fn init(&mut self, _engine: &mut Engine) {}
    fn record(&mut self, frame: &mut Frame, _engine: &mut Engine) {
        let swapchain_image = frame.get_swapchain_image().clone();

        let color = vec4(0.05, 0.05, 0.08, 1.0);

        let builder = frame.get_builder_mut();

        builder
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Float(color.to_array()),
                ..ClearColorImageInfo::image(swapchain_image)
            })
            .unwrap();
    }
}
