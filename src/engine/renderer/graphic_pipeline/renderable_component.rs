use super::uniform::object_uniform::ObjectUniform;


/// Any object uniform whos inner component will act as the rendered object.
/// The component implementing this is not the renderable component itself, so maybe better naming ?
pub trait RenderableComponent: ObjectUniform {
    fn mesh_id(component: &Self::FromComponent) -> u64;
    fn set_uniform_buffer_index(component: &mut Self::FromComponent, index: usize);
    fn uniform_buffer_index(component: &Self::FromComponent) -> usize;
}