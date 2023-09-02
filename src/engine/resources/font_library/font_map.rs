use std::collections::HashMap;

use super::glyph_coords::GlyphData;

/// Maps characters to glyph coordinates.
pub struct FontMap {
    map: HashMap<char, GlyphData>,
    font_atlas_id: u32,
}

impl FontMap {
    pub fn new(font_atlas_id: u32, map: HashMap<char, GlyphData>) -> Self {
        Self {
            map,
            font_atlas_id,
        }
    }

    pub fn insert(&mut self, character: char, coords: GlyphData) {
        self.map.insert(character, coords);
    }

    pub fn get(&self, character: char) -> Option<GlyphData> {
        self.map.get(&character).map(|c| *c)
    }

    pub fn font_atlas_id(&self) -> u32 {
        self.font_atlas_id
    }
}