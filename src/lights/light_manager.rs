use std::collections::HashMap;
use getset::{Getters};

use glam::{IVec3, Vec3, Vec4, vec4};

use crate::{
    common::{LocalLight, Updateable}, lights::light::{AnimationType, Light}, physics::Physics,
};

#[derive(Getters)]
pub struct LightManager {
    lights: HashMap<u32, Light>,
    #[getset(get = "pub with_prefix")]
    size: usize,
}

impl LightManager {
    pub fn new(size: usize) -> Self {
        let lights: HashMap<u32, Light> = HashMap::with_capacity(size);

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

        self.lights.entry(light_id).or_insert(light);
    }

    pub fn get_lights_as_array(&self) -> Vec<LocalLight> {
        self.lights.clone().into_values().collect::<Vec<Light>>().into_iter().map(|light| {
            LocalLight {
                pos_radius: light.get_position().extend(0.0).to_array(),
                color_intensity: light.get_color_intensity().to_array() 
            }
        }).collect()
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

    pub fn update_lights(&mut self, player_position: Vec3) {
        self.lights.retain(|_, light| {
            let lifetime = light.get_lifetime();
            match lifetime {
                None => {
                    light.apply_position_animation(player_position);
                    light.apply_color_animation();

                    return true;
                }
                Some(lifetime) => {
                    if lifetime <= 0.0 { return false; };

                    light.apply_position_animation(player_position);
                    light.apply_color_animation();
                    light.decay();

                    return true;
                }
            }
        });
    }
}

