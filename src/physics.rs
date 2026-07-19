use crate::{
    common::HasEntity,
    entity::Entity,
    engine::Engine,
    world::World
};
use glam::{
    Mat4, Vec3, Vec4Swizzles, vec3, vec4
};

pub struct Physics {
    gravity: f32,
    gravity_direction: Vec3,
    skin_width: f32
}


impl Physics {
    pub fn new(gravity: f32) -> Self {
        Physics {
            gravity,
            gravity_direction: Vec3::NEG_Y,
            skin_width: 0.01
        }
    }
}

impl Physics {
    pub fn step(&self, entities: &mut [&mut dyn HasEntity], engine: &Engine, world: &World) {
        for entity in entities.iter_mut() {
            entity.entity_mut().reset_normal_force();
            if engine.get_flags().get_gravity_state(){
                let mass = entity.entity().get_mass();
                entity.entity_mut().add_applied_force(mass * self.gravity * self.gravity_direction);
            }

            let entity_velocity = entity.entity().get_velocity();

            self.resolve_axis(entity.entity_mut(), engine, Vec3::X, world.get_dimensions_as_arr(), entity_velocity.x * (engine.get_delta_time() as f32 / 1000.0));
            self.resolve_axis(entity.entity_mut(), engine, Vec3::Y, world.get_dimensions_as_arr(), entity_velocity.y * (engine.get_delta_time() as f32 / 1000.0));
            self.resolve_axis(entity.entity_mut(), engine, Vec3::Z, world.get_dimensions_as_arr(), entity_velocity.z * (engine.get_delta_time() as f32 / 1000.0));
        }
    }
}

impl Physics {
    fn resolve_axis(&self, entity: &mut Entity, engine: &Engine, axis: Vec3, resolution: [u32; 3], delta: f32){
        if delta == 0.0 { 
            entity.increment_position(axis * delta);    
            return; 
        }

        let move_direction = delta.signum();
        let move_distance = delta.abs();

        let corners: Mat4 = self.calculate_corners(entity, axis, move_direction);

        let corners = [corners.x_axis, corners.y_axis, corners.z_axis, corners.w_axis, vec4(corners.x_axis.w, corners.y_axis.w, corners.z_axis.w, corners.w_axis.w)];

        let ray_direction = axis * move_direction;

        let mut closest_distance = move_distance;
        let mut any_hit = false;

        for corner in corners {
            debug_assert!(ray_direction.abs().cmpge(Vec3::ONE).any(), "weird");
            let hit = engine.ray_hit_world(corner.xyz(), ray_direction, move_distance, resolution);

            if hit.hit != 0 {
                any_hit = true;
                closest_distance = closest_distance.min(hit.distance);
            }
        }

        let safe_distance = 0_f32.max(closest_distance - self.skin_width);

        if !any_hit {
            entity.increment_position(axis * delta);
            return;
        }

        entity.increment_position(move_direction * axis * safe_distance);

        let hit_normal = axis * -move_direction;

        self.collision(entity, axis, hit_normal);
    }


    fn calculate_corners(&self, entity: &Entity, axis: Vec3, direction: f32) -> Mat4 {
        let half_size = entity.get_size() * 0.5;

        let mut face_center = entity.get_position();

        match axis {
            Vec3::X => {face_center.x += direction * half_size.x},
            Vec3::Y => {face_center.y += direction * half_size.y},
            Vec3::Z => {face_center.z += direction * half_size.z},
            _ => {}
        };

        let (mut axis_a, mut axis_b) = match axis {
            Vec3::X => {(Vec3::Y, Vec3::Z)}
            Vec3::Y => {(Vec3::X, Vec3::Z)},
            Vec3::Z => {(Vec3::X, Vec3::Y)},
            _ => {(Vec3::ZERO, Vec3::ZERO)}
        }; 

        axis_a = half_size.dot(axis_a) - self.skin_width * axis_a;
        axis_b = half_size.dot(axis_b) - self.skin_width * axis_b;

        Mat4::from_cols(
            (face_center + axis_a + axis_b).extend(face_center.x), 
            (face_center + axis_a - axis_b).extend(face_center.y), 
            (face_center - axis_a + axis_b).extend(face_center.z), 
            (face_center - axis_a - axis_b).extend(0.0),
        )
    }

    fn collision(&self, entity: &mut Entity, axis: Vec3, normal: Vec3) {
        let incoming_vel = entity.get_velocity();
        let normal_incoming_vel = incoming_vel.abs().dot(axis) * normal;
        debug_assert!((incoming_vel + normal_incoming_vel).abs().cmple(vec3(1000000.0, 1000000.0, 1000000.0)).any(), "what the fuck");
        entity.set_velocity(incoming_vel + normal_incoming_vel);

        if axis == Vec3::Y && normal.y > 0_f32 {
            entity.add_normal_force(normal * entity.get_mass() * self.gravity);
        } else {
            let push = entity.get_applied_force().dot(normal);

            if push < 0_f32 {
                entity.add_normal_force(normal * -push);
            }
        }
    }
}
