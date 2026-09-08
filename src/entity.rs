use dear_imgui_reflect::ImGuiReflect;
use glam::Vec3;
use getset::{CopyGetters, Setters};
use rapier3d::{prelude::ColliderHandle, control::{KinematicCharacterController, CharacterLength, CharacterAutostep}};

use crate::common::{AABB, Updateable};
use crate::physics::Physics;

const MAX_GROUND_SPEED: f32 = 7.0;
const GROUND_ACCEL: f32 = 10.0;
const AIR_ACCEL: f32 = 2.0;
const GROUND_FRICTION: f32 = 10.0;
const STOP_SPEED: f32 = 1.5;
const GRAVITY: f32 = -9.81;

#[derive(ImGuiReflect, CopyGetters, Setters, Debug)]
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
    #[getset(get_copy = "pub with_prefix")]
    acceleration: glam::Vec3,

    #[getset(get_copy = "pub with_prefix")]
    net_force: glam::Vec3,
    normal_force: glam::Vec3,
    #[getset(get_copy = "pub with_prefix")]
    applied_force: glam::Vec3,

    #[getset(get_copy = "pub with_prefix", set = "pub")]
    grounded: bool,

    #[getset(get_copy = "pub with_prefix", set = "pub")]
    r_yaw: f32,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    r_pitch: f32,

    #[imgui(skip)]
    collider_handle: ColliderHandle,
    #[imgui(skip)]
    controller: KinematicCharacterController,

    #[imgui(skip)]
    wish_dir: glam::Vec3,
    #[imgui(skip)]
    wish_speed: f32,
}

impl Entity {
    pub fn new(mass: f32, size: glam::Vec3, position: glam::Vec3, physics: &mut Physics) -> Self {
        let collider_handle =
            physics.create_capsule_collider(size.y * 0.5, size.x * 0.5, position);

        let mut controller = KinematicCharacterController::default();
        controller.offset = CharacterLength::Absolute(0.01);
        controller.autostep = Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(2.0),
            min_width: CharacterLength::Absolute(0.1),
            include_dynamic_bodies: true,
        });
        controller.snap_to_ground = Some(CharacterLength::Absolute(0.3));

        Entity {
            mass,
            size,
            position,
            velocity: glam::Vec3::ZERO,
            acceleration: glam::Vec3::ZERO,
            net_force: glam::Vec3::ZERO,
            normal_force: glam::Vec3::ZERO,
            applied_force: glam::Vec3::ZERO,
            grounded: false,
            r_yaw: 0.0,
            r_pitch: 0.0,
            collider_handle,
            controller,
            wish_dir: glam::Vec3::ZERO,
            wish_speed: 0.0
        }
    }

    pub fn add_applied_force(&mut self, force: glam::Vec3) {
        self.applied_force += force;
    }

    pub fn add_normal_force(&mut self, force: glam::Vec3) {
        self.normal_force += force;
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

    fn calculate_acceleration(&mut self) {
        self.net_force += self.applied_force + self.normal_force;
        self.acceleration = self.net_force / self.mass;
    }

    fn calculate_velocity(&mut self, delta_time: f32) {
        self.velocity += self.acceleration * delta_time;
    }

    fn apply_ground_friction(&mut self, dt: f32) {
        if !self.grounded {
            return;
        }
        let speed = Vec3::new(self.velocity.x, 0.0, self.velocity.z).length();
        if speed < 1e-4 {
            return;
        }
        let control = speed.max(STOP_SPEED);
        let drop = control * GROUND_FRICTION * dt;
        let new_speed = (speed - drop).max(0.0);
        let scale = new_speed / speed;
        self.velocity.x *= scale;
        self.velocity.z *= scale;
    }

    fn accelerate(&mut self, wish_dir: Vec3, wish_speed: f32, accel: f32, dt: f32) {
        if wish_dir == Vec3::ZERO || wish_speed <= 0.0 {
            return;
        }
        let current_speed = Vec3::new(self.velocity.x, 0.0, self.velocity.z).dot(wish_dir);
        let add_speed = wish_speed - current_speed;
        if add_speed <= 0.0 {
            return;
        }
        let accel_speed = (accel * wish_speed * dt).min(add_speed);
        self.velocity.x += wish_dir.x * accel_speed;
        self.velocity.z += wish_dir.z * accel_speed;
    }

    pub fn set_wish_move(&mut self, wish_dir: Vec3, wish_speed: f32) {
        self.wish_dir = wish_dir;
        self.wish_speed = wish_speed;
    }

    pub fn start_jump(&mut self, _up: Vec3, _frames: u32) {

        let weight = self.mass * 9.81;
        let jump_force = 24.0 * weight;

        self.applied_force += jump_force * Vec3::Y;
    }
}

impl Updateable for Entity {
    fn update(&mut self, delta_time: u128, physics: &mut Physics) {
        let dt = delta_time as f32 / 1000.0;

        self.add_applied_force(Vec3::new(0.0, GRAVITY * self.mass, 0.0));

        self.calculate_acceleration();

        self.reset_applied_force();
        self.reset_normal_force();
        self.reset_net_force();

        self.calculate_velocity(dt);

        let accel = if self.grounded { GROUND_ACCEL } else { AIR_ACCEL };
        if self.grounded {
            self.apply_ground_friction(dt);
        }
        self.accelerate(self.wish_dir, self.wish_speed, accel, dt);

        let desired = self.velocity * dt + 0.5 * self.acceleration * dt.powi(2);
        let (effective, grounded, collision_normals) =
            physics.move_entity(dt, &self.controller, self.collider_handle, desired);

        self.grounded = grounded;
        if grounded {
            self.normal_force = Vec3::new(0.0, self.mass * 9.81, 0.0);
        } else {
            self.normal_force = Vec3::ZERO;
        }
        if grounded && self.velocity.y < 0.0 {
            self.velocity.y = 0.0;
        }

        for normal in &collision_normals {
            let n = normal.normalize_or_zero();
            let vel_into_surface = self.velocity.dot(n);
            if vel_into_surface < 0.0 {
                self.velocity -= n * vel_into_surface;
            }
        }

        physics.apply_collider_movement(self.collider_handle, effective);
        self.position = physics.collider_translation(self.collider_handle);

        self.wish_dir = Vec3::ZERO;
        self.wish_speed = 0.0;
    }
}

impl AABB for Entity {
    fn aabb(&self) -> [Vec3; 2] {
        self.gen_aabb(self.position)
    }

    fn gen_aabb(&self, position: Vec3) -> [Vec3; 2] {
        let size = self.size;
        let width = size.x * 0.5;
        let depth = size.z * 0.5;
        let height = size.y;

        let min = glam::vec3(position.x - width, position.y - height, position.z - depth);
        let max = glam::vec3(position.x + width, position.y, position.z + depth);

        [min, max]
    }
}