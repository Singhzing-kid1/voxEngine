use super::Transition;
use crate::engine::{Engine, Frame};
use dyn_clone::{DynClone, clone_trait_object};

#[cfg(feature = "imgui")]
use crate::engine::Debug;

#[cfg(feature = "imgui")]
use dear_imgui_reflect::Inspector;

pub trait Scene {
    fn on_enter(&mut self, _engine: &mut Engine);

    fn on_exit(&mut self, _engine: &mut Engine);

    fn on_unpause(&mut self, _engine: &mut Engine) {}

    fn handle_input(&mut self, engine: &mut Engine);

    fn update(&mut self, dt: f32, engine: &mut Engine) -> Transition;

    fn render(&mut self, engine: &mut Engine, frame: &mut Frame);

    fn render_as_overlay(&mut self, _engine: &mut Engine, _frame: &mut Frame) {}

    #[cfg(feature = "imgui")]
    fn render_debug(&mut self, _engine: &mut Engine, _debug: &mut Debug) {}

    #[cfg(feature = "imgui")]
    fn imgui_reflect_dyn(&mut self, _ui: &mut Inspector<'_, '_>) -> bool {
        false
    }
}
