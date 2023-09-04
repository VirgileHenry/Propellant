
/// Any object uniform whos inner component will act as the rendered object.
/// The component implementing this is not the renderable component itself, so maybe better naming ?
pub trait RenderableComponent: Sized {
    type FromComponent<Mesh>;
    fn set_uniform<Mesh>(component: &Self::FromComponent<Mesh>, write_to_buf: &mut dyn FnMut(&[Self], usize), instance_count: usize);
    fn mesh_id<Mesh>(component: &Self::FromComponent<Mesh>) -> u64;
    fn set_uniform_buffer_index<Mesh>(component: &mut Self::FromComponent<Mesh>, index: usize);
    fn uniform_buffer_index<Mesh>(component: &Self::FromComponent<Mesh>) -> usize;
    fn instance_count<Mesh>(component: &Self::FromComponent<Mesh>) -> usize;
}