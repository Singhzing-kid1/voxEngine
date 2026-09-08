use sdl3::{EventPump, keyboard::Scancode};

use dear_imgui_reflect::ImGuiReflect;

use crate::{
    camera::Camera,
    common::{HasEntity, Updateable},
    entity::Entity,
    physics::Physics,
};

use getset::Getters;

const MAX_GROUND_SPEED: f32 = 7.0;
const JUMP_FORCE_FRAMES: u32 = 6;

#[allow(unused)]
#[derive(ImGuiReflect, Getters, Debug)]
pub struct Player {
    movement_force: f32,
    reach: i32,
    #[imgui(slider, min = 0.1, max = 1.0)]
    mouse_sensitivity: f32,

    #[getset(get = "pub with_prefix")]
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
        mouse_sensitivity: f32,
        physics: &mut Physics,
    ) -> Self {
        Player {
            movement_force,
            reach,
            mouse_sensitivity,

            camera: Camera::new(fov, near, far, w, h, position),
            entity: Entity::new(mass, size, position, physics),
        }
    }

    pub fn collect_inputs(&mut self, event_pump: &EventPump, x_offset: f32, y_offset: f32) {
        self.camera.add_to_yaw(x_offset * self.mouse_sensitivity);
        self.camera.add_to_pitch(y_offset * self.mouse_sensitivity);

        self.entity.set_r_yaw(self.camera.get_yaw().to_radians());
        self.entity
            .set_r_pitch(self.camera.get_pitch().to_radians());

        let front_horizontal =
            glam::vec3(self.camera.get_front().x, 0.0, self.camera.get_front().z)
                .normalize_or_zero();
        let right_horizontal =
            glam::vec3(self.camera.get_right().x, 0.0, self.camera.get_right().z)
                .normalize_or_zero();

        let mut wish_dir = glam::Vec3::ZERO;

        if event_pump.keyboard_state().is_scancode_pressed(Scancode::W) {
            wish_dir += front_horizontal;
        }

        if event_pump.keyboard_state().is_scancode_pressed(Scancode::S) {
            wish_dir -= front_horizontal;
        }

        if event_pump.keyboard_state().is_scancode_pressed(Scancode::D) {
            wish_dir += right_horizontal;
        }

        if event_pump.keyboard_state().is_scancode_pressed(Scancode::A) {
            wish_dir -= right_horizontal;
        }

        let wish_dir = wish_dir.normalize_or_zero();
        self.entity.set_wish_move(wish_dir, MAX_GROUND_SPEED);

        if event_pump
            .keyboard_state()
            .is_scancode_pressed(Scancode::Space)
            && self.entity.get_grounded()
        {
            self.entity
                .start_jump(self.camera.get_up(), JUMP_FORCE_FRAMES);
        }
    }
}

impl Updateable for Player {
    fn update(&mut self, delta_time: u128, physics: &mut Physics) {
        self.camera.set_camera_position(self.entity.get_position());
        self.entity.update(delta_time, physics);
        self.camera.update(delta_time, physics);
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
