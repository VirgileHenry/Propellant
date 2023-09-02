use std::array::TryFromSliceError;

use crate::engine::errors::{
    PResult,
    loading_errors::LoadingError
};

use super::{vertex::{StaticVertex, VulkanVertex}, Mesh, ToVulkanIntSize};


#[derive(Debug, Clone, Copy)]
pub enum MeshLoadingError {
    FileNotFound,
    InvalidData(TryFromSliceError),
    NotEnoughData,
}

pub trait Loadable {
    fn load_from(data: &[u8]) -> Result<Self, TryFromSliceError> where Self: Sized;
}

impl Loadable for StaticVertex {
    fn load_from(data: &[u8]) -> Result<StaticVertex, TryFromSliceError> {
        let size = std::mem::size_of::<f32>();
        Ok(StaticVertex::new(
            f32::from_ne_bytes(data[0*size..1*size].try_into()?),
            f32::from_ne_bytes(data[1*size..2*size].try_into()?),
            f32::from_ne_bytes(data[2*size..3*size].try_into()?),
            f32::from_ne_bytes(data[3*size..4*size].try_into()?),
            f32::from_ne_bytes(data[4*size..5*size].try_into()?),
            f32::from_ne_bytes(data[5*size..6*size].try_into()?),
            f32::from_ne_bytes(data[6*size..7*size].try_into()?),
            f32::from_ne_bytes(data[7*size..8*size].try_into()?),
        ))
    }
}

impl Loadable for u16 {
    fn load_from(data: &[u8]) -> Result<u16, TryFromSliceError> {
        Ok(u16::from_ne_bytes(data.try_into()?))
    }
}

impl Loadable for u32 {
    fn load_from(data: &[u8]) -> Result<u32, TryFromSliceError> {
        Ok(u32::from_ne_bytes(data.try_into()?))
    }
}

impl<V: Loadable + VulkanVertex, T: Loadable + ToVulkanIntSize> Mesh<V, T> {
    /// Loads a mesh from raw bytes data.
    /// The expected format of the data is :
    /// 
    /// - 1 u32 for the vertex count (4 bytes)
    /// - 1 u32 for the triangle count (4 bytes)
    /// - vertex_count * 32 bytes for the vertices:
    ///     - 3 floats (12 bytes) for the position
    ///     - 3 floats (12 bytes) for the normal
    ///     - 2 floats (8 bytes) for the uv
    /// - triangle_count * 12 bytes for the triangles:
    ///     - u32 (4 bytes) for the first vertex index
    ///     - u32 (4 bytes) for the second vertex index
    ///     - u32 (4 bytes) for the third vertex index
    /// 
    /// So for example, \[0u8; 8\] is an empty mesh (0 vertex, 0 triangle so no data behind.)
    pub fn from_bytes(bytes: &[u8]) -> PResult<Mesh<V, T>> {

        let mut buffer_offset = 0;
        // closure to read from our buffer and increment the offset.
        let mut read_buffer = |size: usize| {
            let result = &bytes[buffer_offset..buffer_offset + size];
            buffer_offset += size;
            result
        };
        let vertex_count: u32 = match read_buffer(4).try_into() {
            Ok(bytes) => u32::from_ne_bytes(bytes),
            Err(_) => return Err(LoadingError::from(MeshLoadingError::NotEnoughData).into()),
        };
        let triangle_count: u32 = match read_buffer(4).try_into() {
            Ok(bytes) => u32::from_ne_bytes(bytes),
            Err(_) => return Err(LoadingError::from(MeshLoadingError::NotEnoughData).into()),
        };
    
        let mut vertices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            match V::load_from(read_buffer(std::mem::size_of::<V>())) {
                Ok(v) => vertices.push(v),
                Err(e) => return Err(LoadingError::from(MeshLoadingError::InvalidData(e)).into()),
            }
        }
    
        let mut triangles = Vec::with_capacity(3 * triangle_count as usize);
        for _ in 0..vertex_count {
            match T::load_from(read_buffer(std::mem::size_of::<T>())) {
                Ok(t) => triangles.push(t),
                Err(e) => return Err(LoadingError::from(MeshLoadingError::InvalidData(e)).into()),
            }
        }
    
        Ok(Mesh::new(vertices, triangles))
    }
}