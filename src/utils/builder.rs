

pub trait HasBuilder {
    type Builder;
    fn builder() -> Self::Builder;
}