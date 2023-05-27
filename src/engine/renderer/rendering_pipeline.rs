use vulkanalia::vk::DeviceV1_0;

pub struct RenderingPipeline {
    pipeline: vulkanalia::vk::Pipeline,
    layout: vulkanalia::vk::PipelineLayout,
}

impl RenderingPipeline {
    pub fn new(
        pipeline: vulkanalia::vk::Pipeline,
        layout: vulkanalia::vk::PipelineLayout,
    ) -> RenderingPipeline {
        RenderingPipeline {
            pipeline,
            layout
        }
    }

    pub fn pipeline(&self) -> vulkanalia::vk::Pipeline {
        self.pipeline
    }

    pub fn layout(&self) -> vulkanalia::vk::PipelineLayout {
        self.layout
    }

    pub fn destroy(&self, device: &vulkanalia::Device) {
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
        }
    }
}