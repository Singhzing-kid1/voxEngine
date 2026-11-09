pub(super) fn voxel_texel_and_bit(
    x: i32,
    y: i32,
    z: i32,
    texel_x: i32,
    texel_y: i32,
) -> (usize, u32) {
    let tx = x / 4;
    let ty = y / 4;
    let tz = z / 8;

    let texel = (tx + ty * texel_x + tz * texel_x * texel_y) as usize;
    let channel = x % 4;
    let bit_in_channel = (y % 4) + (z % 8) * 4;
    let bit = (channel * 32 + bit_in_channel) as u32;

    (texel, bit)
}

pub(super) fn bit_to_local(bit: u32) -> (i32, i32, i32) {
    let channel = (bit / 32) as i32;
    let bit_in_channel = (bit % 32) as i32;

    let local_y = bit_in_channel % 4;
    let local_z = bit_in_channel / 4;

    (channel, local_y, local_z)
}

pub(super) fn local_to_bit(channel: i32, local_y: i32, local_z: i32) -> u32 {
    (channel * 32 + local_y + local_z * 4) as u32
}
