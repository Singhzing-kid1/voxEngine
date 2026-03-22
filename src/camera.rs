use crate::common::Updateable;

pub struct Camera {
    fov: f32,
    near: f32,
    far: f32,

    camera_position: glam::Vec3,
    front: glam::Vec3,
    up: glam::Vec3,
    right: glam::Vec3,

    camera_orientation: glam::Quat,

    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn new(fov: f32, near: f32, far: f32, camera_position: glam::Vec3) -> Self {
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
        }
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn get_near(&self) -> f32 {
        self.near
    }

    pub fn get_far(&self) -> f32 {
        self.far
    }

    pub fn get_front(&self) -> glam::Vec3 {
        self.front
    }

    pub fn get_up(&self) -> glam::Vec3 {
        self.up
    }

    pub fn get_right(&self) -> glam::Vec3 {
        self.right
    }

    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }

    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    pub fn set_yaw(&mut self, value: f32) {
        self.yaw = value;
    }

    pub fn set_pitch(&mut self, value: f32) {
        self.pitch = value;
    }

    pub fn add_to_yaw(&mut self, value: f32) {
        self.yaw += value;
    }

    pub fn add_to_pitch(&mut self, value: f32) {
        self.pitch += value;
    }

    pub fn set_camera_position(&mut self, value: glam::Vec3) {
        self.camera_position = value;
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
    fn update(&mut self, delta_time: u128) {
        let _ = delta_time;
        self.update_orientation();
        self.calculate_right();
        self.calculate_front();
    }
}
