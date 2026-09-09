use crate::common::conversions::{ToRapier, ToRapierVec};
use crate::perlin::FractalNoise;
use getset::CloneGetters;
use glam::{IVec3, Vec3, ivec3};
use rapier3d::prelude::*;
use std::collections::HashMap;

use rayon::prelude::*;

const CHUNK_SIZE: i32 = 32;

#[allow(unused)]
#[derive(CloneGetters)]
pub struct World {
    #[getset(get_clone = "pub with_prefix")]
    world: Vec<u128>,
    height_map: FractalNoise,
    #[getset(get_clone = "pub with_prefix")]
    dimensions: Vec3,
    chunked_coords: HashMap<IVec3, Vec<IVec3>>,
}

impl World {
    pub fn new(seed: u64, dimensions: Vec3) -> Self {
        let height_map = FractalNoise::new(seed, 6, 2.0, 0.5);
        let texel_x = dimensions.x as usize / 4;
        let texel_y = dimensions.y as usize / 4;
        let texel_z = dimensions.z as usize / 8;
        let total = texel_x * texel_y * texel_z;
        let mut world = vec![0u128; total];
        let mut set_voxel = |x: usize, y: usize, z: usize| {
            let tx = x / 4;
            let ty = y / 4;
            let tz = z / 8;
            let texel = tx + ty * texel_x + tz * texel_x * texel_y;
            let channel = x % 4;
            let bit_in_channel = (y % 4) + (z % 8) * 4;
            let bit = channel * 32 + bit_in_channel;
            world[texel] |= 1u128 << bit;
        };
        let noise_frequency = 1.0 / 400.0;
        for x in 0..(dimensions.x as usize) {
            for z in 0..(dimensions.z as usize) {
                let t = height_map.sample(x as f64 * noise_frequency, z as f64 * noise_frequency);
                let height = (dimensions.y as f64 * (t * 0.5 + 0.5)) as usize;
                let height = height.clamp(1, dimensions.y as usize);
                for y in 0..height {
                    set_voxel(x, y, z);
                }
            }
        }

        let mut world_struct = World {
            world,
            height_map,
            dimensions,
            chunked_coords: HashMap::new(),
        };
        world_struct.chunked_coords = world_struct.compute_chunked_surface_coords();
        world_struct
    }

    pub fn get_dimensions_as_arr(&self) -> [u32; 3] {
        [
            self.dimensions.x as u32,
            self.dimensions.y as u32,
            self.dimensions.z as u32,
        ]
    }

    pub fn dimensions_metres(&self) -> (f64, f64, f64) {
        const VOXEL_SIZE_M: f64 = 1.0;
        (
            self.dimensions.x as f64 * VOXEL_SIZE_M,
            self.dimensions.y as f64 * VOXEL_SIZE_M,
            self.dimensions.z as f64 * VOXEL_SIZE_M,
        )
    }

    pub fn get_world_as_u32(&self) -> Vec<u32> {
        self.world
            .iter()
            .flat_map(|v| {
                let bytes = v.to_le_bytes();
                [
                    u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                    u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
                    u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
                    u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
                ]
            })
            .collect()
    }

    pub fn get_voxel(&self, coord: Vec3) -> u128 {
        let texel_x = self.dimensions.x as usize / 4;
        let texel_y = self.dimensions.y as usize / 4;
        let tx = (coord.x as usize) / 4;
        let ty = (coord.y as usize) / 4;
        let tz = (coord.z as usize) / 8;
        let texel = tx + ty * texel_x + tz * texel_x * texel_y;
        let channel = (coord.x as usize) % 4;
        let bit_in_channel = ((coord.y as usize) % 4) + ((coord.z as usize) % 8) * 4;
        let bit = channel * 32 + bit_in_channel;
        (self.world[texel] >> bit) & 1u128
    }
}

impl World {
    fn is_solid_fast(&self, x: i32, y: i32, z: i32, texel_x: i32, texel_y: i32) -> bool {
        let (dx, dy, dz) = (
            self.dimensions.x as i32,
            self.dimensions.y as i32,
            self.dimensions.z as i32,
        );
        if x < 0 || y < 0 || z < 0 || x >= dx || y >= dy || z >= dz {
            return false;
        }
        let tx = x / 4;
        let ty = y / 4;
        let tz = z / 8;
        let texel = (tx + ty * texel_x + tz * texel_x * texel_y) as usize;
        let channel = x % 4;
        let bit_in_channel = (y % 4) + (z % 8) * 4;
        let bit = channel * 32 + bit_in_channel;
        (self.world[texel] >> bit) & 1 != 0
    }

    pub fn is_solid(&self, coord: IVec3) -> bool {
        let texel_x = self.dimensions.x as i32 / 4;
        let texel_y = self.dimensions.y as i32 / 4;
        self.is_solid_fast(coord.x, coord.y, coord.z, texel_x, texel_y)
    }

    fn compute_chunked_surface_coords(&self) -> HashMap<IVec3, Vec<IVec3>> {
        let texel_x = self.dimensions.x as i32 / 4;
        let texel_y = self.dimensions.y as i32 / 4;

        self.world
            .par_iter()
            .enumerate()
            .filter(|(_, word)| **word != 0)
            .fold(
                HashMap::<IVec3, Vec<IVec3>>::new,
                |mut local, (i, &word)| {
                    let i = i as i32;
                    let tx = i % texel_x;
                    let ty = (i / texel_x) % texel_y;
                    let tz = i / (texel_x * texel_y);

                    let mut w = word;
                    while w != 0 {
                        let bit = w.trailing_zeros();
                        w &= w - 1;

                        let channel = (bit / 32) as i32;
                        let bit_in_channel = (bit % 32) as i32;
                        let local_y = bit_in_channel % 4;
                        let local_z = bit_in_channel / 4;

                        let x = tx * 4 + channel;
                        let y = ty * 4 + local_y;
                        let z = tz * 8 + local_z;

                        let neighbor_solid = |dchan: i32,
                                              dy: i32,
                                              dz: i32,
                                              dx: i32,
                                              dz_global: i32|
                         -> bool {
                            let (nchan, nly, nlz) = (channel + dchan, local_y + dy, local_z + dz);
                            if nchan >= 0 && nchan < 4 && nly >= 0 && nly < 4 && nlz >= 0 && nlz < 8
                            {
                                let nbit = nchan * 32 + nly + nlz * 4;
                                (word >> nbit) & 1 != 0
                            } else {
                                self.is_solid_fast(x + dx, y + dy, z + dz_global, texel_x, texel_y)
                            }
                        };

                        let exposed = !neighbor_solid(1, 0, 0, 1, 0)
                            || !neighbor_solid(-1, 0, 0, -1, 0)
                            || !neighbor_solid(0, 1, 0, 0, 0)
                            || !neighbor_solid(0, -1, 0, 0, 0)
                            || !neighbor_solid(0, 0, 1, 0, 1)
                            || !neighbor_solid(0, 0, -1, 0, -1);

                        if exposed {
                            let chunk_key = ivec3(
                                x.div_euclid(CHUNK_SIZE),
                                y.div_euclid(CHUNK_SIZE),
                                z.div_euclid(CHUNK_SIZE),
                            );
                            local.entry(chunk_key).or_default().push(ivec3(x, y, z));
                        }
                    }
                    local
                },
            )
            .reduce(HashMap::new, |mut a, b| {
                for (key, mut coords) in b {
                    a.entry(key).or_default().append(&mut coords);
                }
                a
            })
    }

    pub fn chunk_keys(&self) -> impl Iterator<Item = &IVec3> {
        self.chunked_coords.keys()
    }

    pub fn build_chunk_collider(&self, chunk_key: IVec3) -> Option<Collider> {
        let coords = self.chunked_coords.get(&chunk_key)?;
        if coords.is_empty() {
            return None;
        }
        Some(
            ColliderBuilder::voxels(Vec3::ONE.to_rapier(), &coords.as_slice().to_rapier_vec())
                .build(),
        )
    }
}
