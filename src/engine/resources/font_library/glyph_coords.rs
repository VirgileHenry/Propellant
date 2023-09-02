use fontdue::Metrics;


#[derive(Debug, Clone, Copy)]
/// Coordinates of a glyph in a font atlas.
pub struct GlyphData {
    /// UV of the character in the font texture atlas.
    pub min_uv: glam::Vec2,
    /// UV of the character in the font texture atlas.
    pub max_uv: glam::Vec2,
    /// metrics for building text
    pub metrics: Metrics,
}

impl GlyphData {
    pub fn from(metrics: &fontdue::Metrics, texture_size: (u32, u32), pos: (u32, u32)) -> Self {
        let min_uv = (
            pos.0 as f32 / texture_size.0 as f32,
            pos.1 as f32 / texture_size.1 as f32
        );
        let max_uv = (
            min_uv.0 + metrics.width as f32 / texture_size.0 as f32,
            min_uv.1 + metrics.height as f32 / texture_size.1 as f32
        );
        Self {
            min_uv: glam::Vec2::new(min_uv.0, min_uv.1),
            max_uv: glam::Vec2::new(max_uv.0, max_uv.1),
            metrics: *metrics,
        }
    }
}