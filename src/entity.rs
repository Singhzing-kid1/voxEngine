use dear_imgui_reflect::ImGuiReflect;

use glam::{Vec3};

use getset::{CopyGetters, Setters};

use crate::common::{Updateable, AABB};

#[derive(ImGuiReflect)]
#[derive(CopyGetters, Setters)]
#[derive(Debug)]
pub struct Entity {
    #[imgui(input)]
    #[getset(get_copy = "pub with_prefix")]
    mass: f32,

    #[getset(get_copy = "pub with_prefix")]
    size: glam::Vec3,

    #[getset(get_copy = "pub with_prefix")]
    position: glam::Vec3,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    velocity: glam::Vec3,
    acceleration: glam::Vec3,

    #[getset(get_copy = "pub with_prefix")]
    net_force: glam::Vec3,
    normal_force: glam::Vec3,
    #[getset(get_copy = "pub with_prefix")]
    applied_force: glam::Vec3,

    #[getset(get_copy = "pub with_prefix", set = "pub")]
    r_yaw: f32,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    r_pitch: f32,
}

impl Entity {
    pub fn new(mass: f32, size: glam::Vec3, position: glam::Vec3) -> Self {
        Entity {
            mass,

            size,

            position,
            velocity: glam::Vec3::ZERO,
            acceleration: glam::Vec3::ZERO,

            net_force: glam::Vec3::ZERO,
            normal_force: glam::Vec3::ZERO,
            applied_force: glam::Vec3::ZERO,

            r_yaw: 0.0,
            r_pitch: 0.0,
        }
    }

    pub fn add_applied_force(&mut self, force: glam::Vec3) {
        self.applied_force = self.applied_force + force;
    }

    pub fn add_normal_force(&mut self, force: glam::Vec3) {
        self.normal_force = self.normal_force + force;
    }

    pub fn increment_position(&mut self, value: glam::Vec3) {
        self.position += value;
    }

    pub fn reset_normal_force(&mut self) {
        self.normal_force = Vec3::ZERO;
    }

    pub fn reset_applied_force(&mut self) {
        self.applied_force = Vec3::ZERO;
    }

    pub fn reset_net_force(&mut self) {
        self.net_force = Vec3::ZERO;
    }

}

impl Entity {
    fn calculate_acceleration(&mut self) {
        self.net_force += self.applied_force + self.normal_force;
        self.acceleration = self.net_force / self.mass;
    }

    fn calculate_velocity(&mut self, delta_time: f32) {
        self.velocity += self.acceleration * delta_time;
    }

    fn calculate_position(&mut self, delta_time: f32) {
        self.position += self.velocity * delta_time;
    }
}

impl Updateable for Entity {
    fn update(&mut self, delta_time: u128) {
        let delta_time = delta_time as f32 / 1000.0;
        self.calculate_acceleration();
        self.reset_applied_force();
        self.calculate_velocity(delta_time);
    }
}

impl AABB for Entity {
    fn aabb(&self) -> [Vec3; 2] {
        let position = self.position;
        let size = self.size;

        let width = size.x * 0.5;
        let depth = size.z * 0.5;
        let height = size.y;

        let min = glam::vec3(
            position.x - width,
            position.y - height,
            position.z - depth
        );

        let max = glam::vec3(
            position.x + width,
            position.y,
            position.z + depth
        );

        [min, max]
    }

    fn gen_aabb(&self, position: Vec3) -> [Vec3; 2] {
        let size = self.size;
        
        let width = size.x * 0.5;
        let depth = size.z * 0.5;
        let height = size.y;

        let min = glam::vec3(
            position.x - width,
            position.y - height,
            position.z - depth
        );

        let max = glam::vec3(
            position.x + width,
            position.y,
            position.z + depth
        );

        [min, max]
    }
}
