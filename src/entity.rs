use crate::common::Updateable;

pub struct Entity {
    mass: f32,

    size: glam::Vec3,

    position: glam::Vec3,
    velocity: glam::Vec3,
    acceleration: glam::Vec3,

    net_force: glam::Vec3,
    normal_force: glam::Vec3,
    applied_force: glam::Vec3,

    r_yaw: f32,
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

    pub fn get_mass(&self) -> f32 {
        self.mass
    }

    pub fn get_size(&self) -> glam::Vec3 {
        self.size
    }

    pub fn get_position(&self) -> glam::Vec3 {
        self.position
    }

    pub fn get_velocity(&self) -> glam::Vec3 {
        self.velocity
    }

    pub fn get_applied_force(&self) -> glam::Vec3 {
        self.applied_force
    }

    pub fn get_r_yaw(&self) -> f32 {
        self.r_yaw
    }

    pub fn get_r_pitch(&self) -> f32 {
        self.r_pitch
    }

    pub fn set_r_yaw(&mut self, value: f32) {
        self.r_yaw = value;
    }

    pub fn set_r_pitch(&mut self, value: f32) {
        self.r_pitch = value;
    }
}

impl Entity {
    fn calculate_acceleration(&mut self) {
        self.net_force += self.applied_force + self.normal_force;
        self.acceleration = self.net_force / self.mass;
    }

    fn calculate_velocity(&mut self, delta_time: u128) {
        self.velocity += self.acceleration * delta_time as f32;
    }

    fn calculate_position(&mut self, delta_time: u128) {
        self.position += self.velocity * delta_time as f32;
    }
}

impl Updateable for Entity {
    fn update(&mut self, delta_time: u128) {
        self.calculate_acceleration();
        self.calculate_velocity(delta_time);
        self.calculate_position(delta_time);

        self.normal_force = glam::Vec3::ZERO;
        self.applied_force = glam::Vec3::ZERO;
        self.net_force = glam::Vec3::ZERO;
    }
}
