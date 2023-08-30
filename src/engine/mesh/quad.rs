use super::{vertex::StaticVertex, MeshType};


impl MeshType {
    pub fn flat_quad(size: f32) -> MeshType {
        let vertices = vec![
            StaticVertex::new(-size * 0.5, -0., -size * 0.5, 0., 1., 0., 0., 0.),
            StaticVertex::new(size * 0.5, -0., -size * 0.5, 0., 1., 0., 1., 0.),
            StaticVertex::new(size * 0.5, 0., size * 0.5, 0., 1., 0., 1., 1.),
            StaticVertex::new(-size * 0.5, 0., size * 0.5, 0., 1., 0., 0., 1.),
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];
        MeshType::static_mesh(vertices, indices)
    }
}