use vulkanalia::vk::HasBuilder;

use super::VulkanVertex;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct StaticVertex {
    position: glam::Vec3,
    normal: glam::Vec3,
    uv: glam::Vec2,
}

impl StaticVertex {
    pub fn new(p0: f32, p1: f32, p2: f32, n0: f32, n1: f32, n2: f32, u: f32, v: f32) -> StaticVertex {
        StaticVertex { 
            position: glam::Vec3::new(p0, p1, p2),
            normal: glam::Vec3::new(n0, n1, n2),
            uv: glam::Vec2::new(u, v),
        }
    }
}


impl VulkanVertex for StaticVertex {
    /// Tells to vulkan how to pass this data to the vertex shader.
    fn binding_description() -> vulkanalia::vk::VertexInputBindingDescription {
        vulkanalia::vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<StaticVertex>() as u32)
            .input_rate(vulkanalia::vk::VertexInputRate::VERTEX)
            .build()
    }

    /// Tells the attribute descriptions to vulkan.
    /// There are three of them, as for now: position, normal and uv.
    fn attribute_description() -> Vec<vulkanalia::vk::VertexInputAttributeDescription> {
        let pos = vulkanalia::vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vulkanalia::vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        let norm = vulkanalia::vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vulkanalia::vk::Format::R32G32B32_SFLOAT)
            .offset(std::mem::size_of::<glam::Vec3>() as u32)
            .build();
        let uvs = vulkanalia::vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vulkanalia::vk::Format::R32G32_SFLOAT)
            .offset(2 * std::mem::size_of::<glam::Vec3>() as u32)
            .build();
        vec![pos, norm, uvs]
    }
}