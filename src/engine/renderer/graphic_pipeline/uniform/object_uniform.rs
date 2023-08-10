use std::fmt::Debug;

pub(crate) mod model_uniform;
#[cfg(feature = "ui")]
pub(crate) mod ui_model_uniform;

/// A handle around a UniformBuffer<Any> used as a per object uniform.
/// It acts as the layer between our raw uniform buffer and a more abstract uniform object.
pub trait ObjectUniform: Debug {
    /// The component that is used to get the inner uniform.
    /// This type is the one the pipeline will iterate over.
    type FromComponent;
    fn get_uniform(component: &Self::FromComponent) -> Self;
}
