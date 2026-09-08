pub mod render_compute_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
            #version 460

            const uint DEFAULT = 0; 
            const uint COORD = 1;
            const uint STEPS = 2;
            const uint NORMAL = 3;
            const uint UV = 4;
            const uint DEPTH = 5;

            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

            layout(push_constant) uniform PushConstants {
                mat4 pixelToRay;
                uvec3 voxel_resolution;  // now per-axis: (4000, 4000, 2000)
                uint render_mode;
                float max_ray_length;
                float max_height;
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
            
            vec3 applyFog(vec3 baseColor, float depth) {
                float fog_start = pc.max_ray_length * 0.60;
                float fog_amount = smoothstep(fog_start, pc.max_ray_length, depth);
                vec3 fog_color = vec3(1.0, 1.0, 1.0);
                return mix(baseColor, fog_color, fog_amount);
            }

            vec3 applyFogHeightLimited(vec3 baseColor, float depth, float pointY) {
                if (pointY > pc.max_height) return baseColor; 
                return applyFog(baseColor, depth);
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

                float playerY = origin.y;
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

                tmax = min(tmax, pc.max_ray_length);

                if (tmin > tmax) {
                        if (pc.render_mode == DEFAULT) {
                            float missY = origin.y + dir.y * tmax;
                            color = applyFogHeightLimited(vec3(0.0), tmax, missY);
                        }
                    return false;
                }

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
                    if (steps > max_steps) {
                        if (pc.render_mode == DEFAULT) color = applyFog(vec3(0.0), pc.max_ray_length);
                        return false;
                    }

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

                    if (min(t.x, min(t.y, t.z)) - min(delta.x, min(delta.y, delta.z)) > tmax) {
                        if (pc.render_mode == DEFAULT) {
                            float missY = origin.y + dir.y * tmax;
                            color = applyFogHeightLimited(vec3(0.0), tmax, missY);
                        }
                        return false;
                    }

                    if (any(lessThan(icoord, ivec3(0))) ||
                        any(greaterThanEqual(icoord, ivec3(pc.voxel_resolution)))) {
                        if (pc.render_mode == DEFAULT) {
                            float missY = origin.y + dir.y * tmax;
                            color = applyFogHeightLimited(vec3(0.0), tmax, missY);
                        }
                        return false;
                    }

                    if (readVoxel(uvec3(icoord), texel, texel_coord))
                        break;
                }

                vec3 mask = vec3(0);
                mask[stepped_axis] = 1.0;
                float t_inside = dot(t, mask) - dot(delta, mask);

                switch (pc.render_mode) {
                    case DEFAULT: {
                        vec3 normal = -mask * sgn_dir;
                        color = max(normal.xyz, 0.0) - min(normal.yxz + normal.zyx, 0.0);

                        float depth = (tmin + t_inside) * length(dir);

                        color = applyFog(color, depth);

                        break;
                    }

                    case COORD: {
                        // Normalize each axis by its own resolution
                        color = vec3(icoord) / (res - 1.0);
                        break;
                    }

                    case NORMAL: {
                        vec3 normal = -mask * sgn_dir;
                        color = max(normal.xyz, 0.0) - min(normal.yxz + normal.zyx, 0.0);
                        break;
                    }

                    case UV: {
                        vec3 hit = origin + dir * t_inside;
                        vec3 local_hit = hit - vec3(icoord);
                        vec2 uv;
                        if      (stepped_axis == 0) uv = local_hit.yz;
                        else if (stepped_axis == 1) uv = local_hit.xz;
                        else                        uv = local_hit.xy;
                        color = vec3(uv, 0.0);
                        break;
                    }
                    
                    case DEPTH: {
                        // Normalize depth by the max possible diagonal (in voxels)
                        float max_diagonal = length(res);
                        float depth = (tmin + t_inside) * length(dir);
                        color = vec3(1.0 / (1.0 + depth / max_diagonal))
                            + vec3(0.3, 0.0, 0.7) / 256.0;
                        break;
                    }
                }
                return true;
            }

            void main() {
                ivec2 pixelCoord = ivec2(gl_GlobalInvocationID.xy);
                ivec2 imgSize = imageSize(targetImage);
                if (any(greaterThanEqual(pixelCoord, imgSize))) return;

                vec3 o = pc.pixelToRay[3].xyz;
                vec3 d = normalize(mat3(pc.pixelToRay) * vec3(pixelCoord, 1));

                vec3 color;
                uint steps = 0;
                bool hit = traversal(o, d, color, steps);

                if (pc.render_mode == STEPS) {
                    uint max_steps = pc.voxel_resolution.x + pc.voxel_resolution.y + pc.voxel_resolution.z;
                    color = stepsToInferno(steps, 0u, max_steps);
                    imageStore(targetImage, pixelCoord, vec4(color, 1));
                } else if (hit || pc.render_mode == DEFAULT) {
                    imageStore(targetImage, pixelCoord, vec4(color, 1));
                }
            }
        "
    }
}

pub mod resample_compute_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

            layout(set = 0, binding = 0, rgba8) readonly uniform image2D inputImage;
            layout(set = 0, binding = 1, rgba8) writeonly uniform image2D outputImage;

            void main() {
                ivec2 out_coord = ivec2(gl_GlobalInvocationID.xy);
                ivec2 out_size = imageSize(outputImage);

                if(any(greaterThanEqual(out_coord, out_size))) return;

                ivec2 in_size = imageSize(inputImage);
                ivec2 in_coord = (out_coord * in_size) / out_size;
                in_coord = clamp(in_coord, ivec2(0), in_size - 1);

                imageStore(outputImage, out_coord, imageLoad(inputImage, in_coord));
            }
        "
    }
}

pub mod raycast_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

            layout(push_constant) uniform RayParams {
                vec3 origin;
                vec3 direction;
                float max_distance;
                uvec3 voxel_resolution;
            } ray;

            layout(set = 0, binding = 0, rgba32ui) readonly uniform uimage3D voxelImage;
            layout(set = 1, binding = 0) buffer RaycastResult {
                uint hit;
                float distance;
            } result;

            bool readVoxel(uvec3 coord, inout uvec4 texel, inout ivec3 texel_coord) {
                ivec3 new_texel_coord = ivec3(coord.x / 4, coord.y / 4, coord.z / 8);

                if(new_texel_coord != texel_coord) {
                    texel_coord = new_texel_coord;
                    texel = imageLoad(voxelImage, texel_coord);
                } 
                return bool(texel[coord.x % 4] >> ((coord.y % 4) + (coord.z % 8) * 4) & 1u);
            }

            const float FLT_MAX = 3.4028235e+38;
            const float EPSILON = 1e-4;

            bool traversal(vec3 origin, vec3 dir, inout uint steps) {
                if(length(dir) < 1e-6) return false;

                vec3 res = vec3(ray.voxel_resolution);

                vec3 inv_dir = 1.0 / dir;
                vec3 sgn_dir = sign(dir);
                inv_dir = clamp(inv_dir, vec3(-FLT_MAX), vec3(FLT_MAX));

                vec3 t1 = -origin * inv_dir;
                vec3 t2 = (res - origin) * inv_dir;

                vec3 tmins = min(t1, t2);
                vec3 tmaxs = max(t1, t2);

                float tmin = 0.0;
                float tmax = FLT_MAX;
                for(int i = 0; i < 3; i++){
                    tmin = max(tmin, tmins[i]);
                    tmax = min(tmax, tmaxs[i]);
                }

                tmax = min(tmax, ray.max_distance);

                if(tmin > tmax) return false;

                uint stepped_axis;
                if (tmins.x < tmins.y){
                    if(tmins.x < tmins.z) stepped_axis = 0;
                    else stepped_axis = 2;
                } else {
                    if(tmins.y < tmins.z) stepped_axis = 1;
                    else stepped_axis = 2;
                }

                origin += dir * tmin;


                vec3 snapped = origin + EPSILON * dir;
                ivec3 icoord = ivec3(clamp(floor(snapped), vec3(0), res -1.0));
                ivec3 istep = ivec3(sgn_dir);
                vec3 t = (vec3(icoord) + 0.5 * (1.0 + sgn_dir) - origin) * inv_dir;
                t = max(t, vec3(0.0));
                vec3 delta = inv_dir * sgn_dir;

                uint max_steps = ray.voxel_resolution.x + ray.voxel_resolution.y + ray.voxel_resolution.z + 10u;

                ivec3 texel_coord = ivec3(-1);
                uvec4 texel = uvec4(0);

                bool instantHit = readVoxel(uvec3(icoord), texel, texel_coord);
                if(!instantHit)
                while (true) {
                    steps++;
                    if(steps > max_steps) return false;

                    if(t.x < t.y){
                        if(t.x < t.z) {
                            icoord.x += istep.x;
                            t.x += delta.x;
                            stepped_axis = 0;
                        } else {
                            icoord.z += istep.z;
                            t.z += delta.z;
                            stepped_axis = 2;
                        }
                    } else {
                        if(t.y < t.z) {
                            icoord.y += istep.y;
                            t.y += delta.y;
                            stepped_axis = 1;
                        } else {
                            icoord.z += istep.z;
                            t.z += delta.z;
                            stepped_axis = 2;
                        }
                    }

                    if (min(t.x, min(t.y, t.z)) > tmax) return false;

                    if(any(lessThan(icoord, ivec3(0))) ||
                    any(greaterThanEqual(icoord, ivec3(ray.voxel_resolution))))
                    return false;

                    if(readVoxel(uvec3(icoord), texel, texel_coord)) {
                        result.distance = min(t.x, min(t.y, t.z));
                        break;
                    }
                }

                return true;
            }

            void main(){
                vec3 o = ray.origin;
                vec3 d = normalize(ray.direction);

                uint steps = 0;
                result.hit = traversal(o, d, steps) ? 1u : 0u;
            }       
        "
    }
}
