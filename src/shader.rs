pub mod render_compute_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/raymarch.comp"
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