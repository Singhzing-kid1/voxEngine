use crate::{
    debug::Debug, engine::{Engine, Frame}
};

use dear_imgui_reflect::ImGuiReflect;

pub enum Transition  {
    None,
    Switch(Box<dyn Scene>),
    Pause(Box<dyn Scene>),
    Unpause,
    Quit
}


pub trait Scene {
    fn on_enter(&mut self, _engine: &mut Engine) {}

    fn on_exit(&mut self, _engine: &mut Engine) {}

    fn on_unpause(&mut self, _engine: &mut Engine) {}

    fn handle_input(&mut self, engine: &mut Engine);
    
    fn update(&mut self, dt: f32, engine: &mut Engine) -> Transition;

    fn render(&mut self, engine: &mut Engine, frame: &mut Frame);

    fn render_as_overlay(&mut self, _engine: &mut Engine, _frame: &mut Frame) {}

    fn render_debug(&mut self, _engine: &mut Engine, _debug: &mut Debug) {}

    fn imgui_reflect_dyn(&mut self, _ui: &dear_imgui_rs::Ui) -> bool {
        false
    }
}


pub struct SceneManager {
    current: Box<dyn Scene>,
    overlay: Option<Box<dyn Scene>>
}

impl SceneManager {
    pub fn new(mut initial: Box<dyn Scene>, engine: &mut Engine) -> Self {
        initial.on_enter(engine);

        SceneManager { current: initial, overlay: None}
    }

    pub fn handle_input(&mut self, engine: &mut Engine) {
        self.current.handle_input(engine);
    }

    pub fn update(&mut self, dt: f32, engine: &mut Engine) -> bool {
        match self.current.update(dt, engine) {
            Transition::None => true,
            Transition::Switch(mut next) => {
                self.current.on_exit(engine);
                next.on_enter(engine);
                self.current = next;
                true
            }
            Transition::Pause(mut overlay) => {
                overlay.on_enter(engine);
                self.overlay = Some(overlay);
                true
            }
            Transition::Unpause => {
                self.overlay.as_mut().unwrap().on_exit(engine);
                self.overlay = None;
                self.current.on_unpause(engine);
                true
            }
            Transition::Quit => false
        }
    }

    pub fn render(&mut self, engine: &mut Engine) {
        let mut frame = engine.start_frame();
        
        self.current.render(engine, &mut frame);

        match self.overlay.as_mut() {
            None => {}
            Some(scene) => scene.render_as_overlay(engine, &mut frame)
        }
        
        engine.finish_frame(frame);
    }

    pub fn render_debug(&mut self, engine: &mut Engine, debug: &mut Debug) {
        self.current.render_debug(engine, debug);
    }
}