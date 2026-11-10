use getset::{CopyGetters, Setters};

#[cfg(feature = "imgui")]
use dear_imgui_reflect::ImGuiReflect;

#[cfg_attr(feature = "imgui", derive(ImGuiReflect))]
#[derive(Copy, Clone, Default, CopyGetters, Setters)]
pub struct Flags {
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    quit: bool,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    gravity: bool,
    #[getset(get_copy = "pub with_prefix", set = "pub")]
    capture_mouse: bool,
}
