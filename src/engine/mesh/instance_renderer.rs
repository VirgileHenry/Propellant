
/// Equivalent of a mesh renderer, but for instances.
/// Does not store its own buffers, but a reference to the mesh lib's mesh.
#[allow(unused)]
pub struct InstanceRenderer {
    mesh_id: u64,
}