use crate::Mesh;

use super::vertex::Vertex;


impl Mesh {
    pub fn ui_quad() -> Mesh {
        let vertices = vec![
            Vertex::new(0., 0., 0., 0., 0., 0., 0., 0.),
            Vertex::new(1., 0., 0., 0., 0., 0., 1., 0.),
            Vertex::new(1., 1., 0., 0., 0., 0., 1., 1.),
            Vertex::new(0., 1., 0., 0., 0., 0., 0., 1.),
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];
        Mesh::new(vertices, indices)
    }
}