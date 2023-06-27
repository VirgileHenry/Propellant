use vulkanalia::vk::DeviceV1_0;

pub struct GraphicPipelineCreationState {
    pub stages: Vec<vulkanalia::vk::PipelineShaderStageCreateInfo>,
    pub vertex_input_state: vulkanalia::vk::PipelineVertexInputStateCreateInfo,
    pub vertex_binding_description: Vec<vulkanalia::vk::VertexInputBindingDescription>,
    pub vertex_attribute_description: Vec<vulkanalia::vk::VertexInputAttributeDescription>,
    pub input_assembly_state: vulkanalia::vk::PipelineInputAssemblyStateCreateInfo,
    pub rasterization_state: vulkanalia::vk::PipelineRasterizationStateCreateInfo,
    pub multisample_state: vulkanalia::vk::PipelineMultisampleStateCreateInfo,
    pub color_blend_state: vulkanalia::vk::PipelineColorBlendStateCreateInfo,
    pub color_blend_attachments: Vec<vulkanalia::vk::PipelineColorBlendAttachmentState>,
}

impl GraphicPipelineCreationState {
    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        unsafe {
            for stage in self.stages.drain(..) {
                vk_device.destroy_shader_module(stage.module, None);
            }
        }
    }
}