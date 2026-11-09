mod scene;
mod scene_manager;

//re-exports
pub use scene::Scene;
pub use scene_manager::SceneManager;

pub enum Transition {
    None,
    Switch(Box<dyn Scene>),
    Pause(Box<dyn Scene>),
    Unpause,
    Quit,
}
