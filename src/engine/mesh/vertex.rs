pub(crate) mod static_vertex;
pub(crate) mod skeletal_vertex;

pub(crate) use self::static_vertex::StaticVertex;


pub trait VulkanVertex {
    fn binding_description() -> vulkanalia::vk::VertexInputBindingDescription;
    fn attribute_description() -> Vec<vulkanalia::vk::VertexInputAttributeDescription>;
}