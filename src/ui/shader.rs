pub(super) mod vertex {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 pos;
            layout(location = 1) in vec4 color;

            layout(push_constant) uniform PC {
                vec2 screen_size;
            } pc;

            layout(location = 0) out vec2 v_uv;
            layout(location = 1) out vec4 v_color;

            void main() {
                int corner = gl_VertexIndex & 3;
                v_uv = vec2(
                    float(((corner + 1) >> 1) & 1),
                    float((corner >> 1) & 1)
                );

                gl_Position = vec4(pos / pc.screen_size * 2.0 - 1.0, 0.0, 1.0);
                v_color = color;
            }
        ",
    }
}

pub(super) mod fragment {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 460

            layout(set = 0, binding = 0) uniform sampler2D tex;

            layout(location = 0) in vec2 v_uv;
            layout(location = 1) in vec4 v_color;

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = v_color * texture(tex, v_uv);
            }
        ",
    }
}
