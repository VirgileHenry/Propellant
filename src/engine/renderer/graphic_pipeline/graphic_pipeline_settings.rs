

pub enum DepthSetting {
    None,
    Read,
    Write,
    ReadWrite,
}

pub struct GraphicPipelineSettings {
    depth: DepthSetting,
}

impl Default for GraphicPipelineSettings {
    fn default() -> Self {
        GraphicPipelineSettings {
            depth: DepthSetting::ReadWrite,
        }
    }
}

impl GraphicPipelineSettings {
    pub fn depth(self, depth: DepthSetting) -> GraphicPipelineSettings {
        Self {
            depth,           
        }
    }
}