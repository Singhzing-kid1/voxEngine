use crate::world::World;

use std::sync::{Arc, Mutex};

use super::helpers;
#[cfg(feature = "imgui")]
use dear_imgui_reflect::ImGuiReflect;
use getset::{CopyGetters, Setters};
use rapier3d::{
    control::KinematicCharacterController,
    glamx::{IVec3, Vec3, ivec3},
    parry::query::DefaultQueryDispatcher,
    prelude::*,
};
use std::collections::HashMap;

const CHUNK_LOAD_RADIUS: i32 = 4;

#[derive(CopyGetters, Setters)]
#[cfg_attr(feature = "imgui", derive(ImGuiReflect))]
pub struct Physics {
    #[cfg_attr(feature = "imgui", imgui(skip))]
    rigid_body_set: RigidBodySet,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    soft_body_set: SoftBodySet,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    collider_set: ColliderSet,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    physics_pipeline: PhysicsPipeline,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    island_manager: IslandManager,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    broad_phase: BroadPhaseBvh,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    narrow_phase: NarrowPhase,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    impulse_joint_set: ImpulseJointSet,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    multibody_joint_set: MultibodyJointSet,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    ccd_solver: CCDSolver,
    #[cfg_attr(feature = "imgui", imgui(skip))]
    integration_parameters: IntegrationParameters,

    #[getset(get_copy = "pub with_prefix")]
    gravity: Vec3,

    #[cfg_attr(feature = "imgui", imgui(skip))]
    loaded_chunks: HashMap<IVec3, ColliderHandle>,
    chunk_size: i32,
}

impl Physics {
    pub fn new(gravity: Vec3, chunk_size: i32) -> Self {
        let physics = Physics {
            rigid_body_set: RigidBodySet::new(),
            soft_body_set: SoftBodySet::new(),
            collider_set: ColliderSet::new(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            integration_parameters: IntegrationParameters::default(),
            gravity,
            loaded_chunks: HashMap::new(),
            chunk_size,
        };
        physics
    }

    pub fn update_loaded_chunks(&mut self, world: &World, player_position: Vec3) {
        let player_chunk = ivec3(
            (player_position.x as i32).div_euclid(self.chunk_size),
            (player_position.y as i32).div_euclid(self.chunk_size),
            (player_position.z as i32).div_euclid(self.chunk_size),
        );

        let collider_set = &mut self.collider_set;
        let island_manager = &mut self.island_manager;
        let rigid_body_set = &mut self.rigid_body_set;
        let soft_body_set = &mut self.soft_body_set;

        self.loaded_chunks.retain(|&chunk_key, &mut handle| {
            let in_range =
                helpers::chebyshev_distance(chunk_key, player_chunk) <= CHUNK_LOAD_RADIUS;
            if !in_range {
                collider_set.remove(handle, island_manager, rigid_body_set, soft_body_set, false);
            }
            in_range
        });

        for &chunk_key in world.chunk_keys() {
            if helpers::chebyshev_distance(chunk_key, player_chunk) <= CHUNK_LOAD_RADIUS
                && !self.loaded_chunks.contains_key(&chunk_key)
            {
                if let Some(collider) = world.build_chunk_collider(chunk_key) {
                    let handle = self.collider_set.insert(collider);
                    self.loaded_chunks.insert(chunk_key, handle);
                }
            }
        }
    }

    pub fn step(&mut self) {
        let mut pipeline = self.physics_pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.soft_body_set,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }

    pub fn create_capsule_collider(
        &mut self,
        half_height: f32,
        radius: f32,
        position: Vec3,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::capsule_y(half_height, radius)
            .translation(position)
            .build();
        self.collider_set.insert(collider)
    }

    pub fn collider_translation(&self, handle: ColliderHandle) -> Vec3 {
        self.collider_set[handle].translation()
    }

    pub fn apply_collider_movement(&mut self, handle: ColliderHandle, movement: Vec3) {
        let collider = &mut self.collider_set[handle];
        let new_pos = collider.translation() + movement;
        collider.set_translation(new_pos);
    }

    pub fn move_entity(
        &self,
        dt: f32,
        controller: &KinematicCharacterController,
        handle: ColliderHandle,
        desired_translation: Vec3,
    ) -> (Vec3, bool, Vec<Vec3>) {
        let collider = &self.collider_set[handle];
        let query_pipeline = self.broad_phase.as_query_pipeline(
            &DefaultQueryDispatcher,
            &self.rigid_body_set,
            &self.collider_set,
            QueryFilter::default().exclude_collider(handle),
        );

        let mut collision_normals = Vec::new();

        let effective_movement = controller.move_shape(
            dt,
            &query_pipeline,
            collider.shape(),
            collider.position(),
            desired_translation,
            |collision| {
                collision_normals.push(collision.hit.normal1);
            },
        );

        (
            effective_movement.translation,
            effective_movement.grounded,
            collision_normals,
        )
    }
}
