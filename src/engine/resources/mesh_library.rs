use std::collections::HashMap;

use crate::{
    engine::{
        window::vulkan::{
            vulkan_buffer::VulkanBuffer,
            transfer_command_manager::TransferCommandManager
        },
        errors::PResult, mesh::vertex::Vertex
    },
    Mesh
};

use vulkanalia::vk::DeviceV1_0;

/// Instance of a mesh on the gpu.
#[derive(Debug)]
pub struct LoadedMesh {
    /// The buffer containing the data
    buffer: VulkanBuffer,
    /// number of indices
    index_count: usize,
    /// number of vertices
    vertex_count: usize,
}

impl LoadedMesh {
    pub fn create(
        mesh: Mesh,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<LoadedMesh> {
        // we will use a single buffer for both vertex and index data.
        // [ VERTEX BUFFER | INDEX BUFFER ]
        // create a staging buffer for the buffer (on CPU / RAM)
        let buffer_size = mesh.vertices().len() as u64 * std::mem::size_of::<Vertex>() as u64 + mesh.triangles().len() as u64 * std::mem::size_of::<u32>() as u64;
        let mut staging_buffer = VulkanBuffer::create(
            vk_instance, vk_device, vk_physical_device,
            buffer_size,
            vulkanalia::vk::BufferUsageFlags::TRANSFER_SRC,
            vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT | vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        // copy the vertex data in the staging buffer
        staging_buffer.map_data(
            vk_device,
            mesh.vertices(),
            0,
        )?;
        // copy the index data in the staging buffer
        staging_buffer.map_data(
            vk_device,
            mesh.triangles(),
            mesh.vertices().len() as usize * std::mem::size_of::<Vertex>() as usize,
        )?;
        // create the buffer on the graphic card itself
        let buffer = VulkanBuffer::create(
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
            buffer.buffer(),
            buffer_size,
        )?;


        Ok(LoadedMesh {
            buffer,
            index_count: mesh.triangles().len(),
            vertex_count: mesh.vertices().len(),
        })
    }

    pub fn bind_mesh(
        &self,
        vk_device: &vulkanalia::Device,
        vk_command_buffer: vulkanalia::vk::CommandBuffer,
    ) {
        let buffers = [self.buffer.buffer()];
        let offset = self.vertex_count as u64 * std::mem::size_of::<Vertex>() as u64;
        unsafe {
            vk_device.cmd_bind_vertex_buffers(vk_command_buffer, 0, &buffers, &[0]);
            vk_device.cmd_bind_index_buffer(vk_command_buffer, self.buffer.buffer(), offset, vulkanalia::vk::IndexType::UINT32);
        }
    }

    pub fn index_count(&self) -> usize {
        self.index_count
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    pub fn destroy(
        mut self,
        vk_device: &vulkanalia::Device
    ) {
        self.buffer.destroy(vk_device);
    }
}

/// A library of meshes that can be loaded and used in the scene.
/// maybe specify this is a vulkan mesh lib?
#[derive(Debug)]
pub struct MeshLibrary {
    loading_queue: HashMap<u64, Mesh>,
    meshes: HashMap<u64, LoadedMesh>,
}

impl MeshLibrary {
    pub fn new() -> MeshLibrary {
        MeshLibrary {
            loading_queue: HashMap::new(),
            meshes: HashMap::new(),
        }
    }

    pub fn register_mesh(&mut self, mesh_id: u64, mesh: Mesh) {
        self.loading_queue.insert(mesh_id, mesh);
    }

    pub fn load_meshes(
        &mut self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<()> {
        for (mesh_id, mesh) in self.loading_queue.drain() {
            let loaded_mesh = LoadedMesh::create(
                mesh,
                vk_instance,
                vk_device,
                vk_physical_device,
                vk_transfer_manager,
            )?;
            self.meshes.insert(mesh_id, loaded_mesh);
        }
        Ok(())
    }

    pub fn loaded_mesh(&self, mesh_id: &u64) -> Option<&LoadedMesh> {
        self.meshes.get(mesh_id)
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device
    ) {
        for (_, mesh) in self.meshes.drain() {
            mesh.destroy(vk_device);
        }
    }
}