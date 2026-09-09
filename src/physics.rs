use crate::{
    common::conversions::{FromRapier, ToRapier},
    world::World,
};
use rapier3d::{
    control::KinematicCharacterController, parry::query::DefaultQueryDispatcher, prelude::*,
};
use std::collections::HashMap;

const CHUNK_LOAD_RADIUS: i32 = 4;

pub struct Physics {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    integration_parameters: IntegrationParameters,
    gravity: glam::Vec3,
    loaded_chunks: HashMap<glam::IVec3, ColliderHandle>,
    chunk_size: i32,
}

impl Physics {
    pub fn new(_world: &World, gravity: glam::Vec3, chunk_size: i32) -> Self {
        let physics = Physics {
            rigid_body_set: RigidBodySet::new(),
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

    pub fn update_loaded_chunks(&mut self, world: &World, player_position: glam::Vec3) {
        let player_chunk = glam::ivec3(
            (player_position.x as i32).div_euclid(self.chunk_size),
            (player_position.y as i32).div_euclid(self.chunk_size),
            (player_position.z as i32).div_euclid(self.chunk_size),
        );

        let collider_set = &mut self.collider_set;
        let island_manager = &mut self.island_manager;
        let rigid_body_set = &mut self.rigid_body_set;

        self.loaded_chunks.retain(|&chunk_key, &mut handle| {
            let in_range = chebyshev_distance(chunk_key, player_chunk) <= CHUNK_LOAD_RADIUS;
            if !in_range {
                collider_set.remove(handle, island_manager, rigid_body_set, false);
            }
            in_range
        });

        for &chunk_key in world.chunk_keys() {
            if chebyshev_distance(chunk_key, player_chunk) <= CHUNK_LOAD_RADIUS
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
        self.physics_pipeline.step(
            self.gravity.to_rapier(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }

    pub fn create_capsule_collider(
        &mut self,
        half_height: f32,
        radius: f32,
        position: glam::Vec3,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::capsule_y(half_height, radius)
            .translation(position.to_rapier())
            .build();
        self.collider_set.insert(collider)
    }

    pub fn collider_translation(&self, handle: ColliderHandle) -> glam::Vec3 {
        self.collider_set[handle].translation().from_rapier()
    }

    pub fn apply_collider_movement(&mut self, handle: ColliderHandle, movement: glam::Vec3) {
        let collider = &mut self.collider_set[handle];
        let new_pos = collider.translation() + movement.to_rapier();
        collider.set_translation(new_pos);
    }

    pub fn move_entity(
        &self,
        dt: f32,
        controller: &KinematicCharacterController,
        handle: ColliderHandle,
        desired_translation: glam::Vec3,
    ) -> (glam::Vec3, bool, Vec<glam::Vec3>) {
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
            desired_translation.to_rapier(),
            |collision| {
                collision_normals.push(collision.hit.normal1.from_rapier());
            },
        );

        (
            effective_movement.translation.from_rapier(),
            effective_movement.grounded,
            collision_normals,
        )
    }
}

fn chebyshev_distance(a: glam::IVec3, b: glam::IVec3) -> i32 {
    (a.x - b.x)
        .abs()
        .max((a.y - b.y).abs())
        .max((a.z - b.z).abs())
}
