use super::{World, helpers};
use rapier3d::{
    glamx::{IVec3, Vec3, ivec3},
    prelude::*,
};
use rayon::prelude::*;
use std::collections::HashMap;

impl World {
    pub fn chunk_keys(&self) -> impl Iterator<Item = &IVec3> {
        self.chunked_coords.keys()
    }

    pub fn build_chunk_collider(&self, chunk_key: IVec3) -> Option<Collider> {
        let coords = self.chunked_coords.get(&chunk_key)?;
        if coords.is_empty() {
            return None;
        }
        Some(ColliderBuilder::voxels(Vec3::ONE, &coords.as_slice()).build())
    }

    pub(super) fn is_solid_fast(&self, x: i32, y: i32, z: i32, texel_x: i32, texel_y: i32) -> bool {
        let (dx, dy, dz) = (
            self.dimensions.x as i32,
            self.dimensions.y as i32,
            self.dimensions.z as i32,
        );

        if x < 0 || y < 0 || z < 0 || x >= dx || y >= dy || z >= dz {
            return false;
        }

        let (texel, bit) = helpers::voxel_texel_and_bit(x, y, z, texel_x, texel_y);

        (self.world[texel] >> bit) & 1 != 0
    }

    pub(super) fn compute_chunked_surface_coords(&self) -> HashMap<IVec3, Vec<IVec3>> {
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

                        let (channel, local_y, local_z) = helpers::bit_to_local(bit);

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
                                let nbit = helpers::local_to_bit(nchan, nly, nlz);
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
                                x.div_euclid(self.collider_chunk_size),
                                y.div_euclid(self.collider_chunk_size),
                                z.div_euclid(self.collider_chunk_size),
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
}
