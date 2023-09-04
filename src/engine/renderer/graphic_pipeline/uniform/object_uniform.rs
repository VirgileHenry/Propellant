use std::fmt::Debug;

pub(crate) mod model_uniform;
#[cfg(feature = "ui")]
pub(crate) mod ui_model_uniform;

/// A handle around a UniformBuffer<Any> used as a per object uniform.
/// It acts as the layer between our raw uniform buffer and a more abstract uniform object.
pub trait ObjectUniform: Debug + Sized {
    /// The component that is used to get the inner uniform.
    /// This type is the one the pipeline will iterate over.
    type FromComponent;
    /// Writes the uniform to the buffer.
    /// The write_to_buf closure allows to write a slice to the buffer,
    /// at this instance location with additionnal offset.
    /// The instance count is how many instance this object wants to render,
    /// and so how many we should write to the buffer.
    fn set_uniform(component: &Self::FromComponent, write_to_buf: &mut dyn FnMut(&[Self], usize), instance_count: usize);
}
