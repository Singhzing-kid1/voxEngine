use super::{Generator, helpers, noise::FractalNoise};
use getset::CloneGetters;
use rapier3d::glamx::{IVec3, Vec3};
use std::collections::HashMap;

#[derive(CloneGetters)]
pub struct World {
    #[getset(get_clone = "pub with_prefix")]
    pub(super) world: Vec<u128>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) dimensions: Vec3,
    pub(super) chunked_coords: HashMap<IVec3, Vec<IVec3>>,
    #[getset(get_clone = "pub with_prefix")]
    pub(super) biomes: Vec<u8>,
    pub(super) collider_chunk_size: i32,
}

impl World {
    pub fn new(
        seed: u64,
        dimensions: Vec3,
        collider_chunk_size: i32,
        _generator: Option<Box<dyn Generator>>,
    ) -> Self {
        let height_map = FractalNoise::new(seed, 5, 2.0, 0.5);
        let temperature_map = FractalNoise::new((seed as i64 - 1).abs() as u64, 5, 2.0, 0.5);
        let moisture_map = FractalNoise::new((seed as i64 + 1).abs() as u64, 5, 2.0, 0.5);

        let texel_x = dimensions.x as usize / 4;
        let texel_y = dimensions.y as usize / 4;
        let texel_z = dimensions.z as usize / 8;

        let total = texel_x * texel_y * texel_z;

        let mut world = vec![0u128; total];
        let mut biomes = vec![0u8; ((dimensions.x * dimensions.z) * 4.0) as usize];

        let mut set_voxel = |x: usize, y: usize, z: usize| {
            let (texel, bit) = helpers::voxel_texel_and_bit(
                x as i32,
                y as i32,
                z as i32,
                texel_x as i32,
                texel_y as i32,
            );

            world[texel] |= 1u128 << bit;
        };

        let mut set_biome =
            |x: usize, y: usize, x_len: usize, step: usize, offset: usize, value: u8| {
                let index = (x * x_len) + y;

                biomes[(index * step) + offset] = value;
            };

        let noise_frequency = 1.0 / 450.0;
        let temperature_moisture_freq = 1.0 / 800.0;
        for x in 0..(dimensions.x as usize) {
            for z in 0..(dimensions.z as usize) {
                let height_factor =
                    height_map.sample(x as f64 * noise_frequency, z as f64 * noise_frequency);
                let temperature_raw = temperature_map.sample(
                    x as f64 * temperature_moisture_freq,
                    z as f64 * temperature_moisture_freq,
                );
                let moisture_raw = moisture_map.sample(
                    x as f64 * temperature_moisture_freq,
                    z as f64 * temperature_moisture_freq,
                );

                let temperature = ((temperature_raw * 0.5 + 0.5) * 255.0) as u8;
                let moisture = ((moisture_raw * 0.5 + 0.5) * 255.0) as u8;
                let height_factor = height_factor * 0.5 + 0.5;

                set_biome(x, z, dimensions.x as usize, 4, 0, temperature as u8);
                set_biome(
                    x,
                    z,
                    dimensions.x as usize,
                    4,
                    1,
                    (height_factor * 255.0) as u8,
                );
                set_biome(x, z, dimensions.x as usize, 4, 2, moisture as u8);
                set_biome(x, z, dimensions.x as usize, 4, 3, 255);

                let height = (dimensions.y as f64 * height_factor) as usize;
                let height = height.clamp(1, dimensions.y as usize);
                for y in 0..height {
                    set_voxel(x, y, z);
                }
            }
        }

        let mut world_struct = World {
            world,
            dimensions,
            chunked_coords: HashMap::new(),
            biomes,
            collider_chunk_size,
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
