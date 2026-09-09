use crate::{common::Updateable, physics::Physics};
use dear_imgui_reflect::ImGuiReflect;
use getset::{CopyGetters, Setters};

#[derive(ImGuiReflect, CopyGetters, Setters, Debug)]
pub struct Camera {
    #[imgui(slider, min = 0.0, max = 200.0)]
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    fov: f32,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    near: f32,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    far: f32,
    #[getset(set = "pub")]
    camera_position: glam::Vec3,
    #[getset(get_copy = "pub with_prefix")]
    front: glam::Vec3,
    #[getset(get_copy = "pub with_prefix")]
    up: glam::Vec3,
    #[getset(get_copy = "pub with_prefix")]
    right: glam::Vec3,
    camera_orientation: glam::Quat,
    #[imgui(slider, min = 0.0, max = 360.0)]
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    yaw: f32,
    #[imgui(slider, min = -89.9, max = 89.9)]
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    pitch: f32,
    dimensions: (u16, u16),
}

impl Camera {
    pub fn new(fov: f32, near: f32, far: f32, w: u16, h: u16, camera_position: glam::Vec3) -> Self {
        Camera {
            fov,
            near,
            far,
            camera_position,
            up: glam::Vec3::Y,
            front: glam::Vec3::NEG_Z,
            right: glam::Vec3::ZERO,
            camera_orientation: glam::Quat::IDENTITY,
            yaw: 0.0,
            pitch: 0.0,
            dimensions: (w, h),
        }
    }

    pub fn add_to_yaw(&mut self, value: f32) {
        self.yaw += value;
    }

    pub fn add_to_pitch(&mut self, value: f32) {
        self.pitch += value;

        if self.pitch > 89.9 {
            self.pitch = 89.9;
        } else if self.pitch < -89.9 {
            self.pitch = -89.9;
        }
    }

    pub fn get_pixel_to_ray_matrix(&self, render_scale: u16) -> glam::Mat4 {
        let (w, h) = self.dimensions;
        let (w, h) = ((w / render_scale) as f32, (h / render_scale) as f32);

        let tan_fov = (self.fov.to_radians() * 0.5).tan();

        let f_y = (h * 0.5) / tan_fov;
        let f_x = f_y;

        let cx = w * 0.5;
        let cy = h * 0.5;

        let k_inv = glam::Mat3::from_cols(
            glam::vec3(1.0 / f_x, 0.0, 0.0),
            glam::vec3(0.0, -1.0 / f_y, 0.0),
            glam::vec3(-cx / f_x, cy / f_y, -1.0),
        );

        let r = glam::Mat3::from_quat(self.camera_orientation);

        let m = r * k_inv;

        glam::Mat4::from_cols(
            glam::vec4(m.col(0).x, m.col(0).y, m.col(0).z, 0.0),
            glam::vec4(m.col(1).x, m.col(1).y, m.col(1).z, 0.0),
            glam::vec4(m.col(2).x, m.col(2).y, m.col(2).z, 0.0),
            glam::vec4(
                self.camera_position.x,
                self.camera_position.y,
                self.camera_position.z,
                1.0,
            ),
        )
    }
}

impl Camera {
    fn update_orientation(&mut self) {
        let r_yaw = self.yaw.to_radians();
        let r_pitch = self.pitch.to_radians();

        let q_yaw = glam::Quat::from_axis_angle(glam::Vec3::Y, r_yaw);
        let q_pitch = glam::Quat::from_axis_angle(glam::Vec3::X, r_pitch);

        self.camera_orientation = (q_yaw * q_pitch).normalize();
    }
    fn calculate_right(&mut self) {
        self.right = self.front.cross(self.up).normalize();
    }
    fn calculate_front(&mut self) {
        self.front = self
            .camera_orientation
            .mul_vec3(glam::Vec3::NEG_Z)
            .normalize();
    }
}

impl Updateable for Camera {
    fn fixed_update(&mut self, _physics: &mut Physics) {}

    fn update(&mut self, _alpha: f32, _physics: &mut Physics) {
        self.update_orientation();
        self.calculate_right();
        self.calculate_front();
    }
}
