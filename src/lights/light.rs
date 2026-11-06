use std::{cell::RefCell, rc::Rc};

use getset::{CopyGetters, Getters, Setters};
use glam::{Vec3, Vec4, vec4};

use crate::{common::HasEntity, lights::light::AnimationType::Follow};

use dear_imgui_reflect::ImGuiReflect;

#[derive(Clone)]
pub enum AnimationType {
    Follow(Rc<RefCell<dyn HasEntity>>),
    Flicker
}

impl ImGuiReflect for AnimationType {
    fn imgui_reflect(&mut self, inspector: &mut dear_imgui_reflect::Inspector<'_, '_>, label: &str) -> bool {
        let ui = inspector.ui();
        let mut changed = false;

        match self {
            AnimationType::Follow(entity) => {
                ui.text(format!("{label}: Follow"));
                ui.with_indent(|| {
                    changed |= entity.borrow_mut().entity_mut().imgui_reflect(inspector, "Entity");
                });
            }
            AnimationType::Flicker => {
                ui.text(format!("{label}: Flicker"));
            }
        }

        changed
    }
}

impl Default for AnimationType {
    fn default() -> Self {
        AnimationType::Flicker
    }
}

#[derive(Clone, Default, CopyGetters, Setters, Getters)]
#[derive(ImGuiReflect)]
pub struct Light {
    #[getset(get_copy = "pub with_prefix")]
    position: Vec3,
    #[getset(get_copy = "pub with_prefix")]
    color_intensity: Vec4,

    #[getset(get_copy = "pub with_prefix")]
    lifetime: Option<f64>,

    position_animation: Option<AnimationType>,
    color_animation: Option<AnimationType>,
}

impl Light {
    pub fn new(
        position: Vec3,
        color_intensity: Vec4,
        lifetime: Option<f64>,
        position_animation: Option<AnimationType>,
        color_animation: Option<AnimationType>,
    ) -> Self {
        Self {
            position,
            color_intensity,

            lifetime,

            position_animation,
            color_animation,
        }
    }
}

impl Light {
    pub fn apply_position_animation(&mut self) {
        match &mut self.position_animation {
            None => {}
            Some(AnimationType::Follow(entity)) => {
                self.position = entity.borrow().entity().get_position();
            }
            _ => {}
        }
    }

    pub fn apply_color_animation(&mut self) {
        match &mut self.color_animation {
            None => {}
            Some(_color_animation) => {}
        }
    }

    pub fn decay(&mut self) {
        let current_lifetime = self.lifetime.as_ref().unwrap();
        let intensity = self.color_intensity.w;
        
        let percentage_decrease = intensity / *current_lifetime as f32;


        self.lifetime = Some(*current_lifetime - 1.0);
        self.color_intensity = self.color_intensity - vec4(0.0, 0.0, 0.0, percentage_decrease as f32);

    }
}
