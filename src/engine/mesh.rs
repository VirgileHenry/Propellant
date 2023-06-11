use self::vertex::Vertex;


#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<u32>, // maybe u16 mesh to save memory ?
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, triangles: Vec<u32>) -> Mesh {
        Mesh { vertices, triangles }
    }

    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn triangles(&self) -> &Vec<u32> {
        &self.triangles
    }

}

pub(crate) mod cube;
pub(crate) mod mesh_renderer;
pub(crate) mod mesh_renderer_builder;
pub(crate) mod instance_mesh_lib;
pub(crate) mod instance_mesh_lib_builder;
pub(crate) mod instance_renderer;
pub(crate) mod vertex;