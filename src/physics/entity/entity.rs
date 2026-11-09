use dear_imgui_reflect::ImGuiReflect;
use getset::{CopyGetters, Setters};
use rapier3d::{
    control::{CharacterAutostep, CharacterLength, KinematicCharacterController},
    glamx::Vec3,
    prelude::ColliderHandle,
};

use crate::{common::Updateable, physics::Physics};

#[derive(Copy, Clone, ImGuiReflect, CopyGetters, Setters, Debug)]
pub struct Entity {
    #[imgui(input)]
    #[getset(get_copy = "pub with_prefix")]
    pub(super) mass: f32,

    #[getset(get_copy = "pub with_prefix")]
    pub(super) size: Vec3,

    #[getset(get_copy = "pub with_prefix")]
    pub(super) position: Vec3,
    #[getset(get_copy = "pub with_prefix")]
    pub(super) prev_position: Vec3,

    #[getset(get_copy = "pub with_prefix", set = "pub")]
    pub(super) velocity: Vec3,
    #[getset(get_copy = "pub with_prefix")]
    pub(super) acceleration: Vec3,

    #[getset(get_copy = "pub with_prefix")]
    pub(super) net_force: Vec3,
    pub(super) normal_force: Vec3,
    #[getset(get_copy = "pub with_prefix")]
    pub(super) applied_force: Vec3,

    #[getset(get_copy = "pub with_prefix", set = "pub")]
    pub(super) grounded: bool,

    #[getset(get_copy = "pub with_prefix", set = "pub")]
    pub(super) r_yaw: f32,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    pub(super) r_pitch: f32,

    #[imgui(skip)]
    pub(super) collider_handle: ColliderHandle,
    #[imgui(skip)]
    pub(super) controller: KinematicCharacterController,

    pub(super) wish_dir: Vec3,
    pub(super) wish_speed: f32,
    pub(super) jump_requested: bool,

    pub(super) ground_accel: f32,
    pub(super) air_accel: f32,
    pub(super) ground_friction: f32,
    pub(super) stop_speed: f32,
}

impl Entity {
    pub fn new(mass: f32, size: Vec3, position: Vec3, physics: &mut Physics) -> Self {
        let collider_handle = physics.create_capsule_collider(size.y * 0.5, size.x * 0.5, position);

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
            prev_position: position,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            net_force: Vec3::ZERO,
            normal_force: Vec3::ZERO,
            applied_force: Vec3::ZERO,
            grounded: false,
            r_yaw: 0.0,
            r_pitch: 0.0,
            collider_handle,
            controller,
            wish_dir: Vec3::ZERO,
            wish_speed: 0.0,
            jump_requested: false,
            ground_accel: 10.0,
            air_accel: 2.0,
            ground_friction: 10.0,
            stop_speed: 1.5,
        }
    }
}

impl Updateable for Entity {
    fn fixed_update(&mut self, physics: &mut Physics) {
        let dt = crate::common::TICK_RATE;

        self.prev_position = self.position;

        if self.jump_requested && self.grounded {
            self.velocity.y += 3.0 * 9.81 * 0.3; // derived from v = a * t = (3F / m) * t = (3ma / m) * t = (3a) * t
        }
        self.jump_requested = false;

        self.add_applied_force(physics.get_gravity() * self.mass);

        self.calculate_acceleration();

        self.reset_applied_force();
        self.reset_normal_force();
        self.reset_net_force();

        self.calculate_velocity(dt);

        let accel = if self.grounded {
            self.ground_accel
        } else {
            self.air_accel
        };
        if self.grounded {
            self.apply_ground_friction(self.stop_speed, self.ground_friction, dt);
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
    }

    fn update(&mut self, _alpha: f32, _physics: &mut Physics) {}
}
