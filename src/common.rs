use glam::Vec3;
use std::fmt::Debug;
use vulkano::buffer::BufferContents;

use crate::{entity::Entity, physics::Physics};

pub(crate) const TICK_RATE: f32 = 1.0/60.0; // 1 / fps -> matches rapier internals
pub(crate) const MAX_FRAME_TIME: f32 = 0.25;

#[derive(BufferContents)]
#[repr(C)]
pub struct RayHit {
    pub hit: u32,
    pub distance: f32,
}

pub trait Updateable {
    fn fixed_update(&mut self, physics: &mut Physics);

    fn update(&mut self, alpha: f32, physics: &mut Physics);
}

pub trait HasEntity: Debug {
    fn entity(&self) -> &Entity;
    fn entity_mut(&mut self) -> &mut Entity;
}

pub trait AABB {
    fn aabb(&self) -> [Vec3; 2];
    fn gen_aabb(&self, position: Vec3) -> [Vec3; 2];
}

pub(crate) mod conversions {
    use glam::{IVec3, Vec3, ivec3, vec3};
    use rapier3d::glamx::{IVec3 as RIVec3, Vec3 as RVec3};
    pub trait FromRapier {
        type Output;

        fn from_rapier(self) -> Self::Output;
    }

    pub trait ToRapier {
        type Output;

        fn to_rapier(self) -> Self::Output;
    }

    pub trait ToRapierVec {
        type Output;

        fn to_rapier_vec(&self) -> Vec<Self::Output>;
    }

    impl ToRapier for Vec3 {
        type Output = RVec3;

        fn to_rapier(self) -> Self::Output {
            RVec3::new(self.x, self.y, self.z)
        }
    }

    impl ToRapier for IVec3 {
        type Output = RIVec3;

        fn to_rapier(self) -> Self::Output {
            RIVec3::new(self.x, self.y, self.z)
        }
    }

    impl ToRapierVec for [Vec3] {
        type Output = RVec3;

        fn to_rapier_vec(&self) -> Vec<Self::Output> {
            self.iter().copied().map(ToRapier::to_rapier).collect()
        }
    }

    impl ToRapierVec for [IVec3] {
        type Output = RIVec3;

        fn to_rapier_vec(&self) -> Vec<Self::Output> {
            self.iter().copied().map(ToRapier::to_rapier).collect()
        }
    }

    impl FromRapier for RVec3 {
        type Output = Vec3;

        fn from_rapier(self) -> Self::Output {
            vec3(self.x, self.y, self.z)
        }
    }

    impl FromRapier for RIVec3 {
        type Output = IVec3;

        fn from_rapier(self) -> Self::Output {
            ivec3(self.x, self.y, self.z)
        }
    }
}
