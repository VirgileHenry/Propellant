use std::collections::HashMap;

use crate::engine::errors::{PResult, PropellantError};

use self::{font_map::FontMap, glyph_coords::GlyphData};

use super::texture_library::TextureLibrary;

pub(crate) mod font_map;
pub(crate) mod glyph_coords;

const ATLAS_CHAR_SIZE: u32 = 16;
const MAX_LOADED_CHAR: u32 = ATLAS_CHAR_SIZE * ATLAS_CHAR_SIZE;


pub struct FontLibrary {
    fonts: HashMap<u32, FontMap>,
}

impl FontLibrary {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    pub fn load_font(&mut self, id: u64, font: &[u8], texture_lib: &mut TextureLibrary) -> PResult<u32> {

        // some param to play with at some point
        let cell_size = 128;
        let glyph_size = 64.;

        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default())
            .or(Err(PropellantError::NoResources))?;

        let mut texture = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(ATLAS_CHAR_SIZE * cell_size, ATLAS_CHAR_SIZE * cell_size);
        let mut glyphs = HashMap::new();

        // rasterize every pixel in the texture
        for i in 0..MAX_LOADED_CHAR {
            let c = match std::char::from_u32(i) {
                Some(c) => c,
                None => return Err(PropellantError::Custom("Failed to load char from number {i}".to_string())),
            };
            let (metrics, bitmap) = font.rasterize(c, glyph_size);

            let (x, y) = (i % ATLAS_CHAR_SIZE * cell_size, i / ATLAS_CHAR_SIZE * cell_size);
            
            for i in 0..metrics.width {
                for j in 0..metrics.height {
                    let pixel = bitmap[j * metrics.width + i];
                    let pixel = image::Rgba([pixel, pixel, pixel, pixel]);
                    texture.put_pixel(x + i as u32, y + j as u32, pixel);
                }
            }
            glyphs.insert(c, GlyphData::from(
                &metrics,
                (ATLAS_CHAR_SIZE * cell_size, ATLAS_CHAR_SIZE * cell_size),
                (x, y)
            ));
        }

        let atlas_id = texture_lib.register_built_texture(id, texture)?;

        let font_map = FontMap::new(atlas_id, glyphs);
        self.fonts.insert(atlas_id, font_map);

        Ok(atlas_id)
    }
}