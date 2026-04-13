pub mod camera;
pub mod common;
pub mod engine;
pub mod entity;
pub mod player;
pub mod world;
pub mod debug;
pub mod perlin;

use engine::Engine;
use engine::Flags;
use player::Player;
use std::time;
use world::World;
use debug::Debug;
use crate::common::Updateable;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
            #version 460

            const uint COORD = 0;
            const uint STEPS = 1;
            const uint NORMAL = 2;
            const uint UV = 3;
            const uint DEPTH = 4;

            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

            layout(push_constant) uniform PushConstants {
                mat4 pixelToRay;
                uvec3 voxel_resolution;  // now per-axis: (4000, 4000, 2000)
                uint render_mode;
            } pc;

            layout(set = 0, binding = 0, rgba8) writeonly uniform image2D targetImage;
            layout(set = 1, binding = 0, rgba32ui) readonly uniform uimage3D voxelImage;

            // https://www.shadertoy.com/view/WlfXRN
            vec3 inferno(float t) {
                const vec3 c0 = vec3(0.0002189403691192265, 0.001651004631001012, -0.01948089843709184);
                const vec3 c1 = vec3(0.1065134194856116, 0.5639564367884091, 3.932712388889277);
                const vec3 c2 = vec3(11.60249308247187, -3.972853965665698, -15.9423941062914);
                const vec3 c3 = vec3(-41.70399613139459, 17.43639888205313, 44.35414519872813);
                const vec3 c4 = vec3(77.162935699427, -33.40235894210092, -81.80730925738993);
                const vec3 c5 = vec3(-71.31942824499214, 32.62606426397723, 73.20951985803202);
                const vec3 c6 = vec3(25.13112622477341, -12.24266895238567, -23.07032500287172);
                return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));
            }

            vec3 stepsToInferno(uint steps, uint start, uint end) {
                float t = float(int(steps - start)) / float(end - start);
                return inferno(clamp(t, 0.0, 1.0));
            }

            bool readVoxel(uvec3 coord, inout uvec4 texel, inout ivec3 texel_coord) {
                ivec3 new_texel_coord = ivec3(coord.x / 4, coord.y / 4, coord.z / 8);
                if (new_texel_coord != texel_coord) {
                    texel_coord = new_texel_coord;
                    texel = imageLoad(voxelImage, texel_coord);
                }
                return bool(texel[coord.x % 4] >> ((coord.y % 4) + (coord.z % 8) * 4) & 1u);
            }

            const float FLT_MAX = 3.4028235e+38;

            bool traversal(vec3 origin, vec3 dir, out vec3 color, inout uint steps) {

                if (length(dir) < 1e-6) return false;

                vec3 res = vec3(pc.voxel_resolution);  // (4000, 4000, 2000)

                vec3 inv_dir = 1.0 / dir;
                vec3 sgn_dir = sign(dir);
                inv_dir = clamp(inv_dir, vec3(-FLT_MAX), vec3(FLT_MAX));

                vec3 t1 = -origin * inv_dir;
                vec3 t2 = (res - origin) * inv_dir;

                vec3 tmins = min(t1, t2);
                vec3 tmaxs = max(t1, t2);

                float tmin = 0.0;
                float tmax = FLT_MAX;
                for (int i = 0; i < 3; i++) {
                    tmin = max(tmin, tmins[i]);
                    tmax = min(tmax, tmaxs[i]);
                }

                if (tmin > tmax) return false;

                uint stepped_axis;
                if (tmins.x < tmins.y) {
                    if (tmins.x < tmins.z) stepped_axis = 0;
                    else                   stepped_axis = 2;
                } else {
                    if (tmins.y < tmins.z) stepped_axis = 1;
                    else                   stepped_axis = 2;
                }

                origin += dir * tmin;

                ivec3 icoord = ivec3(clamp(floor(origin), vec3(0), res - 1.0));
                ivec3 istep  = ivec3(sgn_dir);
                vec3  t      = (vec3(icoord) + 0.5 * (1.0 + sgn_dir) - origin) * inv_dir;
                vec3  delta  = inv_dir * sgn_dir;

                // Max steps = longest possible diagonal across the volume
                uint max_steps = pc.voxel_resolution.x + pc.voxel_resolution.y + pc.voxel_resolution.z + 10u;

                ivec3 texel_coord = ivec3(-1);
                uvec4 texel       = uvec4(0);

                bool instantHit = readVoxel(uvec3(icoord), texel, texel_coord);
                if (!instantHit)
                while (true) {
                    steps++;
                    if (steps > max_steps) return false;

                    if (t.x < t.y) {
                        if (t.x < t.z) {
                            icoord.x += istep.x;
                            t.x += delta.x;
                            stepped_axis = 0;
                        } else {
                            icoord.z += istep.z;
                            t.z += delta.z;
                            stepped_axis = 2;
                        }
                    } else {
                        if (t.y < t.z) {
                            icoord.y += istep.y;
                            t.y += delta.y;
                            stepped_axis = 1;
                        } else {
                            icoord.z += istep.z;
                            t.z += delta.z;
                            stepped_axis = 2;
                        }
                    }

                    if (any(lessThan(icoord, ivec3(0))) ||
                        any(greaterThanEqual(icoord, ivec3(pc.voxel_resolution))))
                        return false;

                    if (readVoxel(uvec3(icoord), texel, texel_coord))
                        break;
                }

                vec3 mask = vec3(0);
                mask[stepped_axis] = 1.0;
                float t_inside = dot(t, mask) - dot(delta, mask);

                switch (pc.render_mode) {
                    case COORD:
                        // Normalize each axis by its own resolution
                        color = vec3(icoord) / (res - 1.0);
                        break;
                    case NORMAL:
                        vec3 normal = -mask * sgn_dir;
                        color = max(normal.xyz, 0.0) - min(normal.yxz + normal.zyx, 0.0);
                        break;
                    case UV:
                        vec3 hit = origin + dir * t_inside;
                        vec3 local_hit = hit - vec3(icoord);
                        vec2 uv;
                        if      (stepped_axis == 0) uv = local_hit.yz;
                        else if (stepped_axis == 1) uv = local_hit.xz;
                        else                        uv = local_hit.xy;
                        color = vec3(uv, 0.0);
                        break;
                    case DEPTH:
                        // Normalize depth by the max possible diagonal (in voxels)
                        float max_diagonal = length(res);
                        float depth = (tmin + t_inside) * length(dir);
                        color = vec3(1.0 / (1.0 + depth / max_diagonal))
                            + vec3(0.3, 0.0, 0.7) / 256.0;
                        break;
                }
                return true;
            }

            void main() {
                ivec2 pixelCoord = ivec2(gl_GlobalInvocationID.xy);
                ivec2 imgSize = imageSize(targetImage);

                if (any(greaterThanEqual(pixelCoord, imgSize))) return;

                vec3 o = pc.pixelToRay[3].xyz;
                vec3 d = mat3(pc.pixelToRay) * vec3(pixelCoord, 1);

                vec3 color;
                uint steps = 0;
                if (traversal(o, d, color, steps) && pc.render_mode != STEPS) {
                    imageStore(targetImage, pixelCoord, vec4(color, 1));
                }
                if (pc.render_mode == STEPS) {
                    uint max_steps = pc.voxel_resolution.x + pc.voxel_resolution.y + pc.voxel_resolution.z;
                    color = stepsToInferno(steps, 0u, max_steps);
                    imageStore(targetImage, pixelCoord, vec4(color, 1));
                }
            }
        "
    }
}

mod rs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

            layout(set = 0, binding = 0, rgba8) readonly uniform image2D inputImage;
            layout(set = 0, binding = 1, rgba8) writeonly uniform image2D outputImage;

            void main() {
                ivec2 outputCoords = ivec2(gl_GlobalInvocationID.xy);

                ivec2 inputSize = imageSize(inputImage);
                ivec2 outputSize = imageSize(outputImage);

                if (outputCoords.x >= outputSize.x || outputCoords.y >= outputSize.y) {
                    return;
                }

                vec2 ratio = vec2(inputSize) / vec2(outputSize);
                vec2 inputRegionMin = vec2(outputCoords) * ratio;
                vec2 inputRegionMax = vec2(outputCoords + 1) * ratio;

                vec4 areaWeightedSum = vec4(0.0);
                for (int y = int(floor(inputRegionMin.y)); y < int(ceil(inputRegionMax.y)); ++y) {
                    for (int x = int(floor(inputRegionMin.x)); x < int(ceil(inputRegionMax.x)); ++x) {
                        vec2 pixelMin = vec2(x, y);
                        vec2 pixelMax = pixelMin + 1.0;
                        vec2 overlapMin = max(pixelMin, inputRegionMin);
                        vec2 overlapMax = min(pixelMax, inputRegionMax);
                        vec2 overlapSize = max(overlapMax - overlapMin, 0.0);
                        float overlapArea = overlapSize.x * overlapSize.y;

                        areaWeightedSum += imageLoad(inputImage, ivec2(x, y)) * overlapArea;
                    }
                }

                vec2 inputRegionSize = inputRegionMax - inputRegionMin;
                float totalArea = inputRegionSize.x * inputRegionSize.y;

                vec4 areaWeightedAvg = areaWeightedSum / totalArea;
                imageStore(outputImage, outputCoords, areaWeightedAvg);
            }
        "
    }
}

fn main() {
    let mut flags = Flags::new();

    flags.set_capture_mouse_state(true);

    let mut engine = Engine::new(
        "vox engine using rust",
        1920,
        1080,
        time::Instant::now(),
        flags,
    );
    println!("initialized engine");


    let mut debug = Debug::new(&engine);
    println!("initialized debug ui");

    let (w, h) = engine.get_dimensions();

    let mut player = Player::new(
        90.0,
        0.1,
        1000.0,
        100.0,
        1000.0,
        10,
        w,
        h,
        glam::vec3(4.0, 550.0, 9.0),
        glam::Vec3::ONE,
    );

    println!("start world generation");
    let world = World::new(416120398);

    
    engine.send_world_data(world.get_world_as_u32(), world.dimensions());
    println!("sent world data to gpu");


    engine.toggle_mouse(engine.get_flags().get_capture_mouse_state());

    while !engine.get_flags().get_quit_state() {
        engine.event_handling();

        player.collect_inputs(
            engine.get_event(),
            engine.get_x_offset(),
            engine.get_y_offset(),
        );

        player.update(engine.get_delta_time());

        engine.render(player.get_camera().get_pixel_to_ray_matrix(), world.dimensions());
        debug.render(&mut engine, &mut player);
        engine.present();
    }
}
