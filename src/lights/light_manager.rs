#[cfg(feature = "imgui")]
use dear_imgui_reflect::ImGuiReflect;
use getset::Getters;
use std::collections::HashMap;

use rapier3d::glamx::{Vec3, Vec4};

use crate::{
    common::{LocalLight, Updateable},
    physics::Physics,
};

use super::{AnimationType, light::Light};

#[derive(Getters)]
#[cfg_attr(feature = "imgui", derive(ImGuiReflect))]
pub struct LightManager {
    lights: HashMap<String, Light>,
    #[getset(get = "pub with_prefix")]
    size: usize,
}

impl LightManager {
    pub fn new(size: usize) -> Self {
        let lights: HashMap<String, Light> = HashMap::with_capacity(size);

        Self { lights, size }
    }
}

impl LightManager {
    pub fn add_light(
        &mut self,
        position: Vec3,
        color_intensity: Vec4,
        lifetime: Option<f64>,
        position_animation: Option<AnimationType>,
        color_animation: Option<AnimationType>,
        world_extents: [u32; 3],
    ) {
        let light_id = (world_extents[0]
            * ((position.z as u32 * world_extents[1]) + position.x as u32))
            + position.y as u32;

        let light = Light::new(
            position,
            color_intensity,
            lifetime,
            position_animation,
            color_animation,
        );

        self.lights.entry(format!("{light_id}")).or_insert(light);
    }

    pub fn get_lights_as_array(&self) -> Vec<LocalLight> {
        self.lights
            .clone()
            .into_values()
            .collect::<Vec<Light>>()
            .into_iter()
            .map(|light| LocalLight {
                pos_radius: light.get_position().extend(0.0).to_array(),
                color_intensity: light.get_color_intensity().to_array(),
            })
            .collect()
    }

    pub fn get_amount_of_active_lights(&self) -> u32 {
        self.lights
            .clone()
            .into_values()
            .collect::<Vec<Light>>()
            .into_iter()
            .filter(|light| {
                let lifetime = light.get_lifetime();
                match lifetime {
                    None => {
                        return true;
                    }
                    Some(lifetime) => {
                        if lifetime > 0.0 {
                            return true;
                        };
                        return false;
                    }
                }
            })
            .count() as u32
    }
}

impl Updateable for LightManager {
    fn update(&mut self, alpha: f32, _physics: &mut Physics) {
        self.lights.retain(|_, light| {
            let lifetime = light.get_lifetime();
            match lifetime {
                None => {
                    light.apply_position_animation(alpha);
                    light.apply_color_animation();

                    true
                }
                Some(lifetime) => {
                    if lifetime <= 0f64 {
                        return false;
                    }

                    light.apply_position_animation(alpha);
                    light.apply_color_animation();
                    light.decay();

                    true
                }
            }
        });
    }
}
