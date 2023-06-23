use std::{
    io::{
        BufReader,
        Read
    },
    array::TryFromSliceError
};

use crate::{Mesh, engine::errors::{PResult, loading_errors::LoadingError}};

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
            f32::from_ne_bytes(data[0..size].try_into()?),
            f32::from_ne_bytes(data[size..2*size].try_into()?),
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
    pub fn load_mesh(path: &str) -> PResult<Mesh> {
        let mut reader = BufReader::new(
            match std::fs::File::open(path) {
                Ok(file) => file,
                Err(_) => return Err(LoadingError::from(MeshLoadingError::FileNotFound).into()),
            }
        );
        let mut u32_buffer = [0u8; 4];
        let vertex_count: usize = match reader.read_exact(&mut u32_buffer) {
            Err(_) => return Err(LoadingError::from(MeshLoadingError::InvalidData).into()),
            Ok(_) => match u32::from_ne_bytes(u32_buffer).try_into() {
                Ok(value) => value,
                Err(_) => return Err(LoadingError::from(MeshLoadingError::NotEnoughData).into()),
            },
        };
        let triangle_count: usize = match reader.read_exact(&mut u32_buffer) {
            Err(_) => return Err(LoadingError::from(MeshLoadingError::InvalidData).into()),
            Ok(_) => match u32::from_ne_bytes(u32_buffer).try_into() {
                Ok(value) => value,
                Err(_) => return Err(LoadingError::from(MeshLoadingError::NotEnoughData).into()),
            },
        };
    
        let mut vertices = Vec::with_capacity(vertex_count);
        let mut vertex_buffer = [0u8; std::mem::size_of::<Vertex>()];
        for _ in 0..vertex_count {
            match reader.read_exact(&mut vertex_buffer) {
                Ok(_) => {
                    vertices.push(match Vertex::load_from(&vertex_buffer) {
                        Ok(vert) => vert,
                        Err(e) => {
                            return Err(LoadingError::from(MeshLoadingError::InvalidData).into())
                        },
                    })
                },
                Err(_) => return Err(LoadingError::from(MeshLoadingError::NotEnoughData).into()),
            }
        }
    
        let mut triangles = Vec::with_capacity(3 * triangle_count);
        let mut triangle_buffer = [0u8; std::mem::size_of::<u32>()];
        for _ in 0..3*triangle_count {
            match reader.read_exact(&mut triangle_buffer) {
                Ok(_) => triangles.push(u32::from_ne_bytes(triangle_buffer)),
                Err(_) => return Err(LoadingError::from(MeshLoadingError::NotEnoughData).into()),
            }
        }
    
    
        Ok(Mesh::new(vertices, triangles))
    }
}