pub(crate) use self::vertex::StaticVertex;

use super::errors::PResult;


pub(crate) mod cube;
pub(crate) mod quad;
pub(crate) mod mesh_renderer;
pub(crate) mod sphere;
pub(crate) mod vertex;
pub(crate) mod loader;
#[cfg(feature = "ui")]
pub(crate) mod ui_quad;


pub(crate) type StaticMeshVertexType = StaticVertex;
pub(crate) type StaticMeshTriangleType = u32;

pub type StaticMesh = Mesh<StaticMeshVertexType, StaticMeshTriangleType>;

#[derive(Debug, Clone)]
pub enum MeshType {
    Static(Mesh<StaticMeshVertexType, StaticMeshTriangleType>),
    // Skinned(),
}

impl MeshType {
    pub fn static_mesh(vertices: Vec<StaticMeshVertexType>, triangles: Vec<StaticMeshTriangleType>) -> MeshType {
        MeshType::Static(Mesh::new(vertices, triangles))
    }

    pub fn load_static_mesh(bytes: &[u8]) -> PResult<MeshType> {
        Ok(MeshType::Static(Mesh::from_bytes(bytes)?))
    }

    pub fn buffer_size(&self) -> usize {
        match self {
            MeshType::Static(mesh) => mesh.vertices().len() * std::mem::size_of::<StaticMeshVertexType>() + mesh.triangles().len() * std::mem::size_of::<StaticMeshTriangleType>(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh<V, T> {
    pub vertices: Vec<V>,
    pub triangles: Vec<T>,
}

impl<V, T> Mesh<V, T> {
    pub fn new(vertices: Vec<V>, triangles: Vec<T>) -> Mesh<V, T> {
        Mesh { vertices, triangles }
    }

    pub fn vertices(&self) -> &Vec<V> {
        &self.vertices
    }

    pub fn triangles(&self) -> &Vec<T> {
        &self.triangles
    }

}
