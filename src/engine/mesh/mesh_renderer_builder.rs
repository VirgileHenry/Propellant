use crate::engine::{
    errors::PResult,
    material::Material,
    mesh::vertex::Vertex, 
    window::vulkan::{
        vulkan_buffer::VulkanBuffer,
        transfer_command_manager::TransferCommandManager
    }
};

use super::{Mesh, mesh_renderer::MeshRenderer};



pub struct MeshRendererBuilder {
    mesh: Mesh,
    material: Material,
}

impl MeshRendererBuilder {
    pub fn new(mesh: Mesh, material: Material) -> MeshRendererBuilder {
        MeshRendererBuilder {
            mesh,
            material,
        }
    }

    pub fn build(
        self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<MeshRenderer> {
        // we will use a single buffer for both vertex and index data.
        // [ VERTEX BUFFER | INDEX BUFFER ]
        // create a staging buffer for the buffer (on CPU / RAM)
        let buffer_size = self.mesh.vertices().len() as u64 * std::mem::size_of::<Vertex>() as u64 + self.mesh.triangles().len() as u64 * std::mem::size_of::<u32>() as u64;
        let mut staging_buffer = VulkanBuffer::create(
            vk_instance, vk_device, vk_physical_device,
            buffer_size,
            vulkanalia::vk::BufferUsageFlags::TRANSFER_SRC,
            vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT | vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        // copy the vertex data in the staging buffer
        staging_buffer.map_data(
            vk_device,
            &self.mesh.vertices(),
            0,
        )?;
        // copy the index data in the staging buffer
        staging_buffer.map_data(
            vk_device,
            &self.mesh.triangles(),
            self.mesh.vertices().len() as usize * std::mem::size_of::<Vertex>() as usize,
        )?;
        // create the buffer on the graphic card itself
        let device_buffer = VulkanBuffer::create(
            vk_instance, vk_device, vk_physical_device,
            buffer_size,
            // we need a target buffer that can be used as a vertex buffer, index buffer and transfer destination
            vulkanalia::vk::BufferUsageFlags::TRANSFER_DST | vulkanalia::vk::BufferUsageFlags::VERTEX_BUFFER | vulkanalia::vk::BufferUsageFlags::INDEX_BUFFER,
            vulkanalia::vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        // set the buffer transfer on the queue
        vk_transfer_manager.register_transfer(
            vk_device,
            staging_buffer,
            device_buffer.buffer(),
            buffer_size,
        )?;


        Ok(MeshRenderer::new(
            device_buffer,
            self.mesh.vertices().len(),
            self.mesh.triangles().len(),
            self.material
        ))
    }
}