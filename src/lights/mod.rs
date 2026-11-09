mod light;
mod light_manager;

pub use light_manager::LightManager;

use crate::common::HasEntity;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub enum AnimationType {
    Follow(Rc<RefCell<dyn HasEntity>>),
    Flicker,
}
