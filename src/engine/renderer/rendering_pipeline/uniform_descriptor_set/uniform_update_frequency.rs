

/// Defines how often a uniform should be updated to the vulkan buffer.
#[derive(Debug, Clone)]
pub enum UniformUpdateFrequency {
    /// Uniforms are never updated, and only set once.
    StartOnly,
    /// Uniforms are updated every frame.
    EachFrame,
    /// Uniforms are updated at a fixed rate.
    Timed(Vec<f32>, f32), // (time, rate)
}

impl UniformUpdateFrequency {
    pub fn timed(rate: f32) -> UniformUpdateFrequency {
        UniformUpdateFrequency::Timed(vec![0.], rate)
    }
}