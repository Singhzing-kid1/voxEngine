use crate::perlin::FractalNoise;
use glam::Vec3;

#[allow(unused)]
pub struct World {
    world: Vec<u128>,
    height_map: FractalNoise,
    dimensions: Vec3,
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
        World {
            world,
            height_map,
            dimensions,
        }
    }

    pub fn get_dimensions(&self) -> [u32; 3] {
        [
            self.dimensions.x as u32,
            self.dimensions.y as u32,
            self.dimensions.z as u32,
        ]
    }

    pub fn dimensions_metres(&self) -> (f64, f64, f64) {
        const VOXEL_SIZE_M: f64 = 0.50;
        (
            self.dimensions.x as f64 * VOXEL_SIZE_M,
            self.dimensions.y as f64 * VOXEL_SIZE_M,
            self.dimensions.z as f64 * VOXEL_SIZE_M,
        )
    }

    pub fn get_world(&self) -> Vec<u128> {
        self.world.clone()
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
}

