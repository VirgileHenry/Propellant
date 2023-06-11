use vk_shader_macros::include_glsl;

pub static DEFAULT_FRAG: &'static [u32] = include_glsl!("src/shaders/default.frag");
pub static DEFAULT_VERT: &'static [u32] = include_glsl!("src/shaders/default.vert");
