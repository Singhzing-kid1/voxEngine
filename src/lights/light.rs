use getset::{CopyGetters, Getters, Setters};
use glam::{Vec3, Vec4, vec4};

use crate::lights::light::AnimationType::Follow;

#[derive(Copy, Clone)]
pub enum AnimationType {
    Follow,
}

#[derive(Copy, Clone, CopyGetters, Setters, Getters)]
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
    pub fn apply_position_animation(&mut self, new_position: Vec3) {
        match self.position_animation {
            None => {}
            Some(position_animation) => {
                match position_animation {
                    AnimationType::Follow => {
                        self.position = new_position;
                    }
                }
            }
        }
    }

    pub fn apply_color_animation(&mut self) {
        match self.color_animation {
            None => {}
            Some(color_animation) => {}
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
