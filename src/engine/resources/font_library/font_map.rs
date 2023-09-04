use std::collections::HashMap;

use super::glyph_coords::GlyphData;


#[derive(Debug, Clone)]
/// Maps characters to glyph coordinates.
pub struct FontMap {
    map: HashMap<char, GlyphData>,
    unfound_glyph: GlyphData,
    font_atlas_id: u32,
    glyph_size: f32,
}

impl FontMap {
    pub fn new(font_atlas_id: u32, map: HashMap<char, GlyphData>, unfound_glyph: GlyphData, glyph_size: f32) -> Self {
        Self {
            map,
            unfound_glyph,
            font_atlas_id,
            glyph_size,
        }
    }

    pub fn get(&self, character: char) -> GlyphData {
        match self.map.get(&character) {
            Some(glyph) => *glyph,
            None => self.unfound_glyph,
        }
    }

    pub fn font_atlas_id(&self) -> u32 {
        self.font_atlas_id
    }

    pub fn glyph_size(&self) -> f32 {
        self.glyph_size
    }
}