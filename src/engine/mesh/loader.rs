use std::array::TryFromSliceError;

use crate::{
    Mesh,
    engine::errors::{
        PResult,
        loading_errors::LoadingError
    }
};

use super::vertex::Vertex;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshLoadingError {
    FileNotFound,
    InvalidData,
    NotEnoughData,
}


impl Vertex {
    pub fn load_from(data: &[u8]) -> Result<Vertex, TryFromSliceError> {
        let size = std::mem::size_of::<f32>();
        Ok(Vertex::new(
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

impl Mesh {
    
    pub fn from_bytes(bytes: &[u8]) -> PResult<Mesh> {

        let mut buffer_offset = 0;
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
            match Vertex::load_from(read_buffer(std::mem::size_of::<Vertex>())) {
                Ok(v) => vertices.push(v),
                Err(_) => return Err(LoadingError::from(MeshLoadingError::InvalidData).into()),
            }
        }
    
        let mut triangles = Vec::with_capacity(3 * triangle_count as usize);
        for _ in 0..vertex_count {
            triangles.push(
                u32::from_ne_bytes(match read_buffer(4).try_into() {
                    Ok(bytes) => bytes,
                    Err(_) => return Err(LoadingError::from(MeshLoadingError::InvalidData).into()),
                })
            )
        }
    
        Ok(Mesh::new(vertices, triangles))
    }
}