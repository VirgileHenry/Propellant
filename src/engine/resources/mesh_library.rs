use std::collections::HashMap;

use crate::{
    engine::{
        window::vulkan::{
            vulkan_buffer::VulkanBuffer,
            transfer_command_manager::TransferCommandManager
        },
        errors::PResult, mesh::{vertex::StaticVertex, MeshType, StaticMeshVertexType}
    },
    id
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
    /// type of int used for this mesh
    index_type: vulkanalia::vk::IndexType,
}

impl LoadedMesh {
    pub fn create(
        mesh: MeshType,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<LoadedMesh> {
        // we will use a single buffer for both vertex and index data.
        // [ VERTEX BUFFER | INDEX BUFFER ]
        // create a staging buffer for the buffer (on CPU / RAM)
        let buffer_size = mesh.buffer_size() as u64;
        let mut staging_buffer = VulkanBuffer::create(
            vk_instance, vk_device, vk_physical_device,
            buffer_size,
            vulkanalia::vk::BufferUsageFlags::TRANSFER_SRC,
            vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT | vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        // copy the vertex data in the staging buffer
        match &mesh {
            MeshType::Static(mesh) => staging_buffer.map_data(
                vk_device,
                mesh.vertices(),
                0,
            )?,
        };
        // copy the index data in the staging buffer
        match &mesh {
            MeshType::Static(mesh) => staging_buffer.map_data(
                vk_device,
                mesh.triangles(),
                mesh.vertices().len() * std::mem::size_of::<StaticMeshVertexType>(),
            )?,
        };
        // create the buffer on the graphic card itself
        let buffer = VulkanBuffer::create(
            vk_instance, vk_device, vk_physical_device,
            buffer_size,
            // we need a target buffer that can be used as a vertex buffer, index buffer and transfer destination
            vulkanalia::vk::BufferUsageFlags::TRANSFER_DST | vulkanalia::vk::BufferUsageFlags::VERTEX_BUFFER | vulkanalia::vk::BufferUsageFlags::INDEX_BUFFER,
            vulkanalia::vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        // set the buffer transfer on the queue
        vk_transfer_manager.register_buffer_transfer(
            vk_device,
            staging_buffer,
            buffer.buffer(),
            buffer_size,
        )?;

        let index_count = match &mesh {
            MeshType::Static(mesh) => mesh.triangles().len(),
        };
        let vertex_count = match &mesh {
            MeshType::Static(mesh) => mesh.vertices().len(),
        };

        let index_type = mesh.index_type();

        Ok(LoadedMesh {
            buffer,
            index_count,
            vertex_count,
            index_type,
        })
    }

    pub fn bind_mesh(
        &self,
        vk_device: &vulkanalia::Device,
        vk_command_buffer: vulkanalia::vk::CommandBuffer,
    ) {
        let buffers = [self.buffer.buffer()];
        let offset = self.vertex_count as u64 * std::mem::size_of::<StaticVertex>() as u64;

        unsafe {
            vk_device.cmd_bind_vertex_buffers(vk_command_buffer, 0, &buffers, &[0]);
            vk_device.cmd_bind_index_buffer(vk_command_buffer, self.buffer.buffer(), offset, self.index_type);
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
    loading_queue: HashMap<u64, MeshType>,
    meshes: HashMap<u64, LoadedMesh>,
}

impl MeshLibrary {
    pub fn new() -> MeshLibrary {
        MeshLibrary {
            loading_queue: HashMap::new(),
            meshes: HashMap::new(),
        }
    }

    #[cfg(feature = "ui")]
    pub fn with_ui_quad() -> MeshLibrary {
        let mut result = MeshLibrary::new();
        result.register_mesh(id("ui_quad"), MeshType::ui_quad());
        result
    }

    pub fn register_mesh(&mut self, mesh_id: u64, mesh: MeshType) {
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