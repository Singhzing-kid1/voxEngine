pub mod render_compute_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/raymarch.comp",
        include: ["shaders/include/"]
    }
}

pub mod light_flood_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/light_flood.comp",
        include: ["shaders/include/"]
    }
}

pub mod apply_light_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/apply_light.comp",
        include: ["shaders/include/"]
    }
}

pub mod resample_compute_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/nn_lerp.comp"
    }
}

pub mod raycast_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/raycast.comp"
    }
}

pub mod pause_blur_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/pause_blur.comp"
    }
}
