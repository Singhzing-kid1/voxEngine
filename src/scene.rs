use crate::{
    engine::Engine,
    debug::Debug
};

pub enum Transition  {
    None,
    Switch(Box<dyn Scene>),
    Quit
}

pub trait Scene {
    fn on_enter(&mut self, _engine: &mut Engine) {}

    fn on_exit(&mut self, _engine: &mut Engine) {}

    fn handle_input(&mut self, engine: &mut Engine);
    
    fn update(&mut self, dt: f32, engine: &mut Engine) -> Transition;

    fn render(&mut self, engine: &mut Engine);

    fn render_debug(&mut self, _engine: &mut Engine, _debug: &mut Debug) {}
}


pub struct SceneManager {
    current: Box<dyn Scene>
}

impl SceneManager {
    pub fn new(mut initial: Box<dyn Scene>, engine: &mut Engine) -> Self {
        initial.on_enter(engine);

        SceneManager { current: initial }
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
            Transition::Quit => false
        }
    }

    pub fn render(&mut self, engine: &mut Engine) {
        self.current.render(engine);
    }

    pub fn render_debug(&mut self, engine: &mut Engine, debug: &mut Debug) {
        self.current.render_debug(engine, debug);
    }
}