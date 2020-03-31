use webrender::api::{GlyphDimensions, FontKey, FontInstanceKey, RenderApi};
use webrender::api::units::{LayoutSize};

pub struct LayoutedText<'a> {
    pub text: &'a str,
    pub indices: Vec<u32>,
    pub dimensions: Vec<GlyphDimensions>,
    pub font_key: FontKey,
    pub inst_key: FontInstanceKey,
    pub size: LayoutSize
}

impl<'a> LayoutedText<'a> {
    pub fn new(text: &'a str, font_key: FontKey, inst_key: FontInstanceKey, api: &RenderApi) -> Self {
        let indices: Vec<u32> = api
            .get_glyph_indices(font_key, text)
            .iter()
            .filter_map(|&x| x)
            .collect();
        let dimensions: Vec<GlyphDimensions> = api
            .get_glyph_dimensions(inst_key, indices.clone())
            .iter()
            .filter_map(|&x| x)
            .collect();

        let (size_x, size_y) = dimensions.iter().fold((0.0, 0.0), |(x, y), &g| {
            let dy = (g.height + (g.height - g.top)) as f32;
            (x + g.advance, f32::max(y, dy))
        });

        let size = LayoutSize::new(size_x as f32, size_y as f32);

        LayoutedText {
            text,
            indices,
            dimensions,
            font_key,
            inst_key,
            size
        }
    }
}