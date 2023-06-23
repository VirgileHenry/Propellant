use self::vertex::Vertex;


pub(crate) mod cube;
pub(crate) mod quad;
pub(crate) mod mesh_renderer;
pub(crate) mod sphere;
pub(crate) mod vertex;
pub(crate) mod loader;

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
