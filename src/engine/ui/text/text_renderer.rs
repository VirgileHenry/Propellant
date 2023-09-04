use crate::{engine::{resources::font_library::font_map::FontMap, ui::ui_resolution::UiResolution}, UiTransform};

use super::character_renderer::CharacterRenderer;


/// let's implement this the bad way first?
/// This is super wanky at the moment, we need a system to layout the text.
pub struct UiTextRenderer {
    text: String,
    font: u32,
    characters: Vec<CharacterRenderer>,
    instance_offset: usize,
    font_size: f32,
    font_color: glam::Vec3,
}

impl UiTextRenderer {
    pub fn new(text: String, font: u32, color: glam::Vec3) -> Self {
        Self {
            text,
            font,
            characters: Vec::new(),
            instance_offset: 0,
            font_size: 20.,
            font_color: color,
        }
    }

    pub fn characters(&self) -> &[CharacterRenderer] {
        &self.characters
    }

    pub fn set_instance_offset(&mut self, offset: usize) {
        self.instance_offset = offset;
    }

    pub fn instance_offset(&self) -> usize {
        self.instance_offset
    }

    pub fn font(&self) -> u32 {
        self.font
    }

    pub fn rebuild_text(&mut self, transform: &UiTransform, font: &FontMap, ui_res: UiResolution) {
        self.characters = Vec::with_capacity(self.text.len());
        let mut x = 0.;
        let mut y = self.font_size;
        let parent_pos = transform.get_pos();
        let scale = self.font_size / font.glyph_size();
        for c in self.text.chars() {
            if c == '\n' {
                x = 0.;
                y += self.font_size;
                continue;
            }
            let glyph_data = font.get(c);
            let pos = glam::vec2(
                x + glyph_data.metrics.xmin as f32 * scale,
                y - glyph_data.metrics.ymin as f32 * scale,
            );
            let size = glam::vec2(
                glyph_data.metrics.width as f32 * scale,
                glyph_data.metrics.height as f32 * scale,
            );
            // same math as for child of ui tf
            let scale_factor = ui_res.scale_factor() / parent_pos.size();
            let total_size = size * scale_factor;
            let total_pos = -glam::Vec2::ONE + (pos * scale_factor) * 2. + total_size * glam::vec2(1., -1.);
            let mat = glam::Mat3::from_scale_angle_translation(total_size, 0., total_pos);
            let char_renderer = CharacterRenderer::new(glyph_data.min_uv, glyph_data.max_uv, font.font_atlas_id(), mat, self.font_color);
            self.characters.push(char_renderer);
            x += glyph_data.metrics.advance_width * scale;
        }
    }
}


