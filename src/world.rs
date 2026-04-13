use std::io::{self, Write};
use crate::perlin::FractalNoise;

//TODO make this part of the world struct
const WORLD_X: usize = 2000;
const WORLD_Y: usize = 1000;
const WORLD_Z: usize = 2000;

#[allow(unused)]
pub struct World {
    world: Vec<u128>,
    height_map: FractalNoise,
    dim_x: usize,
    dim_y: usize,
    dim_z: usize,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let dim_x = WORLD_X;
        let dim_y = WORLD_Y;
        let dim_z = WORLD_Z;

        let height_map = FractalNoise::new(seed, 6, 2.0, 0.5);

        let texel_x = dim_x / 4;
        let texel_y = dim_y / 4;
        let texel_z = dim_z / 8;
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

        for x in 0..dim_x {
            for z in 0..dim_z {
                let t = height_map.sample(x as f64 * noise_frequency, z as f64 * noise_frequency);
                let height = (dim_y as f64 * (t * 0.5 + 0.5)) as usize;
                let height = height.clamp(1, dim_y);

                for y in 0..height {
                    set_voxel(x, y, z);

                }
            }
        }
        World { world, height_map, dim_x, dim_y, dim_z }
    }

    pub fn dimensions(&self) -> [u32; 3] {
        [self.dim_x as u32, self.dim_y as u32, self.dim_z as u32]
    }

    pub fn dimensions_metres(&self) -> (f64, f64, f64) {
        const VOXEL_SIZE_M: f64 = 0.50;
        (
            self.dim_x as f64 * VOXEL_SIZE_M,
            self.dim_y as f64 * VOXEL_SIZE_M,
            self.dim_z as f64 * VOXEL_SIZE_M,
        )
    }

    pub fn get_world(&self) -> Vec<u128> {
        self.world.clone()
    }

    pub fn get_world_as_u32(&self) -> Vec<u32> {
        self.world.iter().flat_map(|v| {
            let bytes = v.to_le_bytes();
            [
                u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
                u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
                u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
            ]
        }).collect()
    }
}