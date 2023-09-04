use crate::{engine::renderer::graphic_pipeline::renderable_component::RenderableComponent, id};

use super::text_renderer::UiTextRenderer;


#[repr(C)]
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
/// Renders a single character.
pub struct CharacterRenderer {
    min_uv: glam::Vec2,
    max_uv: glam::Vec2,
    col1: glam::Vec4,
    col2: glam::Vec4,
    col3: glam::Vec4,
    color: glam::Vec3,
    texture_id: u32,
}

impl CharacterRenderer {
    pub fn new(min_uv: glam::Vec2, max_uv: glam::Vec2, font_texture: u32, transform: glam::Mat3, color: glam::Vec3) -> Self {
        CharacterRenderer {
            min_uv,
            max_uv,
            col1: (transform.x_axis, 0.).into(),
            col2: (transform.y_axis, 0.).into(),
            col3: (transform.z_axis, 0.).into(),
            color,
            texture_id: font_texture,
        }
    }
}

impl RenderableComponent for CharacterRenderer {
    type FromComponent<Mesh> = UiTextRenderer;
    
    fn instance_count<Mesh>(component: &Self::FromComponent<Mesh>) -> usize {
        component.characters().len()    
    }

    fn mesh_id<Mesh>(_component: &Self::FromComponent<Mesh>) -> u64 {
        id("ui_quad")
    }

    fn set_uniform<Mesh>(component: &Self::FromComponent<Mesh>, write_to_buf: &mut dyn FnMut(&[Self], usize), instance_count: usize) {
        debug_assert!(instance_count == component.characters().len());
        write_to_buf(&component.characters(), 0);
    }

    fn set_uniform_buffer_index<Mesh>(component: &mut Self::FromComponent<Mesh>, index: usize) {
        component.set_instance_offset(index);
    }

    fn uniform_buffer_index<Mesh>(component: &Self::FromComponent<Mesh>) -> usize {
        component.instance_offset()
    }
}



