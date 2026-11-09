use getset::{CopyGetters, Setters};

use dear_imgui_reflect::ImGuiReflect;

#[derive(Copy, Clone, Default, CopyGetters, Setters, ImGuiReflect)]
pub(crate) struct Flags {
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    quit: bool,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    gravity: bool,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    capture_mouse: bool,
}
