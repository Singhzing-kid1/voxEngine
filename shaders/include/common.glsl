bool readVoxel(uvec3 coord, inout uvec4 texel, inout ivec3 texel_coord) {
    ivec3 new_texel_coord = ivec3(coord.x / 4, coord.y / 4, coord.z / 8);
    if (new_texel_coord != texel_coord) {
        texel_coord = new_texel_coord;
        texel = imageLoad(voxelImage, texel_coord);
    }
    return bool(texel[coord.x % 4] >> ((coord.y % 4) + (coord.z % 8) * 4) & 1u);
}