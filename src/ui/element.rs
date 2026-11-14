use crate::engine::scene::Transition;
use rapier3d::glamx::{Vec2, Vec3};

pub enum Action {
    SceneTransition(Transition),
}

pub trait Element {
    fn set_action(&mut self, action: Action);
    fn set_texture(&mut self, index: u32);
    fn get_tris(&self) -> (Vec<Vec3>, Vec<u32>);
    fn get_action(&self) -> Action;
    fn in_bounds(&self, _position: Vec2) -> bool {
        false
    }
}
