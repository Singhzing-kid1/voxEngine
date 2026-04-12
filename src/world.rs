use crate::perlin::FractalNoise;

pub struct World {
    world: Vec<u128>,
    height_map: FractalNoise,

}

impl World {
    pub fn new(resolution: u32, seed: u64) -> Self {
        let height_map = FractalNoise::new(seed, 6, 2.0, 0.5);


        let res = resolution as usize;
        let texel_dim = res / 4;
        let texel_dim_z = res / 8;
        let total = texel_dim * texel_dim * texel_dim_z;
        let mut world = vec![0u128; total];

        let mut set_voxel = |x: usize, y: usize, z: usize| {
            let tx = x / 4;
            let ty = y / 4;
            let tz = z / 8;
            let texel = tx + ty * texel_dim + tz * texel_dim * texel_dim;

            let channel = x % 4;
            let bit_in_channel = (y % 4) + (z % 8) * 4;
            let bit = channel * 32 + bit_in_channel;

            world[texel] |= 1u128 << bit;
        };

        for x in 0..(resolution / 4) {
            for z in 0..(resolution / 8) {
                let height = (resolution / 4) as f64 * height_map.sample(x as f64 * 0.1, z as f64 * 0.1) * 0.5 + 0.5;
                for y in 1..height as u32 {
                    set_voxel(x as usize, 0, z as usize);
                    set_voxel(x as usize, y as usize, z as usize);
                }
            }   
        }



        World { world, height_map }
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
