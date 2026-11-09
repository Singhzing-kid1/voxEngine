use rapier3d::glamx::IVec3;

pub fn chebyshev_distance(a: IVec3, b: IVec3) -> i32 {
    (a.x - b.x)
        .abs()
        .max((a.y - b.y).abs())
        .max((a.z - b.z).abs())
}
