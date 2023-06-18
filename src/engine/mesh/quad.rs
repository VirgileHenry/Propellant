use crate::Mesh;

use super::vertex::Vertex;


impl Mesh {
    pub fn flat_quad(size: f32) -> Mesh {
        let vertices = vec![
            Vertex::new(-size * 0.5, -0., -size * 0.5, 0., 0., 1., 0., 0.),
            Vertex::new(size * 0.5, -0., -size * 0.5, 0., 0., 1., 1., 0.),
            Vertex::new(size * 0.5, 0., size * 0.5, 0., 0., 1., 1., 1.),
            Vertex::new(-size * 0.5, 0., size * 0.5, 0., 0., 1., 0., 1.),
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];
        Mesh::new(vertices, indices)
    }
}