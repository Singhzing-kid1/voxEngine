pub struct World {
    world: Vec<u128>,
}

impl World {
    pub fn new(resolution: u32) -> Self {
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

        set_voxel(1, 1, 1);
        set_voxel(1, 1, 2);



        World { world }
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
