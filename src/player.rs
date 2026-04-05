use sdl3::EventPump;
use sdl3::keyboard::Scancode;

use crate::camera::Camera;
use crate::common::{HasEntity, Updateable};
use crate::entity::Entity;

pub struct Player {
    movement_force: f32,
    reach: i32,

    camera: Camera,
    entity: Entity,
}

impl Player {
    pub fn new(
        fov: f32,
        near: f32,
        far: f32,
        mass: f32,
        movement_force: f32,
        reach: i32,
        w: u16,
        h: u16,
        position: glam::Vec3,
        size: glam::Vec3,
    ) -> Self {
        Player {
            movement_force,
            reach,

            camera: Camera::new(fov, near, far, w, h, position),
            entity: Entity::new(mass, size, position),
        }
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn get_controller(&self) -> &Entity {
        &self.entity
    }

    pub fn collect_inputs(&mut self, event_pump: &EventPump, x_offset: f32, y_offset: f32) {
        self.camera.add_to_yaw(x_offset);
        self.camera.add_to_pitch(y_offset);

        self.entity.set_r_yaw(self.camera.get_yaw().to_radians());
        self.entity
            .set_r_pitch(self.camera.get_pitch().to_radians());

        let front_horizontal =
            glam::vec3(self.camera.get_front().x, 0.0, self.camera.get_front().z).normalize_or_zero();
        let right_horizontal =
            glam::vec3(self.camera.get_right().x, 0.0, self.camera.get_right().z).normalize_or_zero();

        if event_pump.keyboard_state().is_scancode_pressed(Scancode::W) {
            self.entity
                .add_applied_force(self.movement_force * front_horizontal);
        }

        if event_pump.keyboard_state().is_scancode_pressed(Scancode::S) {
            self.entity
                .add_applied_force(-self.movement_force * front_horizontal);
        }

        if event_pump.keyboard_state().is_scancode_pressed(Scancode::D) {
            self.entity
                .add_applied_force(self.movement_force * right_horizontal);
        }

        if event_pump.keyboard_state().is_scancode_pressed(Scancode::A) {
            self.entity
                .add_applied_force(-self.movement_force * right_horizontal);
        }

        if event_pump
            .keyboard_state()
            .is_scancode_pressed(Scancode::Space)
        {
            let weight = self.entity.get_mass() * 9.81;
            let jump_force = weight + weight * 0.1;

            self.entity
                .add_applied_force(jump_force * self.camera.get_up());
        }
    }
}

impl Updateable for Player {
    fn update(&mut self, delta_time: u128) {
        self.camera.set_camera_position(self.entity.get_position());

        self.entity.update(delta_time);
        self.camera.update(delta_time);
    }
}

impl HasEntity for Player {
    fn entity(&self) -> &Entity {
        &self.entity
    }

    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}
