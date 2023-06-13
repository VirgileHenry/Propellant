use crate::engine::{
    material::Material,
    mesh::vertex::Vertex,
    window::vulkan::vulkan_buffer::VulkanBuffer
};


use vulkanalia::vk::DeviceV1_0;

/// Component to render a Mesh.
pub struct MeshRenderer {
    buffer: VulkanBuffer,
    vertex_count: usize,
    index_count: usize,
    material: Material,
}

impl MeshRenderer {
    pub fn new(buffer: VulkanBuffer, vertex_count: usize, index_count: usize, material: Material) -> MeshRenderer {
        MeshRenderer {
            buffer,
            vertex_count,
            index_count,
            material,
        }
    }

    pub fn register_draw_commands(
        &self, 
        vk_device: &vulkanalia::Device,
        vk_command_buffer: vulkanalia::vk::CommandBuffer,
    ) {
        let vertex_buffers = [self.buffer.buffer()];
        // offset is in bytes
        let offset = self.vertex_count as u64 * std::mem::size_of::<Vertex>() as u64;
        unsafe {
            vk_device.cmd_bind_vertex_buffers(vk_command_buffer, 0, &vertex_buffers, &[0]);
            vk_device.cmd_bind_index_buffer(vk_command_buffer, self.buffer.buffer(), offset, vulkanalia::vk::IndexType::UINT32);
            vk_device.cmd_draw_indexed(vk_command_buffer, self.index_count as u32, 1, 0, 0, 0);
        }
    }

    pub fn pipeline_id(&self) -> u64 {
        self.material.pipeline_id()
    }

    pub fn material(&self) -> &Material {
        &self.material
    } 

    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.buffer.destroy(vk_device);
    }
}