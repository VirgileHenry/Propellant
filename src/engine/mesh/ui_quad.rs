
use super::{vertex::StaticVertex, MeshType};


impl MeshType {
    pub fn ui_quad() -> MeshType {
        let vertices = vec![
            StaticVertex::new(-1., -1., 1., 0., 0., 0., 0., 0.),
            StaticVertex::new( 1., -1., 1., 0., 0., 0., 1., 0.),
            StaticVertex::new( 1.,  1., 1., 0., 0., 0., 1., 1.),
            StaticVertex::new(-1.,  1., 1., 0., 0., 0., 0., 1.),
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];
        MeshType::static_mesh(vertices, indices)
    }
}