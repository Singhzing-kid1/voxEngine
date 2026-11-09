use super::Entity;
use rapier3d::glamx::Vec3;

impl Entity {
    pub fn add_applied_force(&mut self, force: Vec3) {
        self.applied_force += force;
    }

    pub fn add_normal_force(&mut self, force: Vec3) {
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

    pub fn set_wish_move(&mut self, wish_dir: Vec3, wish_speed: f32) {
        self.wish_dir = wish_dir;
        self.wish_speed = wish_speed;
    }

    pub fn request_jump(&mut self) {
        self.jump_requested = true;
    }

    pub fn interpolated_position(&self, alpha: f32) -> Vec3 {
        self.prev_position.lerp(self.position, alpha)
    }
}

impl Entity {
    pub(super) fn calculate_acceleration(&mut self) {
        self.net_force += self.applied_force + self.normal_force;
        self.acceleration = self.net_force / self.mass;
    }

    pub(super) fn calculate_velocity(&mut self, delta_time: f32) {
        self.velocity += self.acceleration * delta_time;
    }

    pub(super) fn apply_ground_friction(&mut self, stop_speed: f32, ground_friction: f32, dt: f32) {
        if !self.grounded {
            return;
        }
        let speed = Vec3::new(self.velocity.x, 0.0, self.velocity.z).length();
        if speed < 1e-4 {
            return;
        }
        let control = speed.max(stop_speed);
        let drop = control * ground_friction * dt;
        let new_speed = (speed - drop).max(0.0);
        let scale = new_speed / speed;
        self.velocity.x *= scale;
        self.velocity.z *= scale;
    }

    pub(super) fn accelerate(&mut self, wish_dir: Vec3, wish_speed: f32, accel: f32, dt: f32) {
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
}
