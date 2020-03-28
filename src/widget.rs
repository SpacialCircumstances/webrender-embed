use webrender::api::*;
use webrender::api::units::*;

trait Widget {
    fn draw(&mut self, builder: &mut DisplayListBuilder, space_clip: SpaceAndClipInfo) -> ();
}

struct Rect {
    area: LayoutRect,
    color: ColorF
}

impl Widget for Rect {
    fn draw(&mut self, builder: &mut DisplayListBuilder, space_clip: SpaceAndClipInfo) -> () {
        let info = CommonItemProperties::new(self.area, space_clip);
        builder.push_rect(&info, self.color);
    }
}

struct Root {
    child: Box<dyn Widget>
}

impl Root {
    pub fn new(child: Box<dyn Widget>) -> Self {
        Root {
            child
        }
    }
}

impl Widget for Root {
    fn draw(&mut self, builder: &mut DisplayListBuilder, space_clip: SpaceAndClipInfo) -> () {
        self.child.draw(builder, space_clip);
    }
}

struct Group {
    children: Vec<Box<dyn Widget>>
}

impl Group {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Self {
        Group {
            children
        }
    }
}

impl Widget for Group {
    fn draw(&mut self, builder: &mut DisplayListBuilder, space_clip: SpaceAndClipInfo) -> () {
        self.children.iter_mut().for_each(|w| w.draw(builder, space_clip));
    }
}

struct LayoutedText<'a> {
    text: &'a str,
    indices: Vec<u32>,
    dimensions: Vec<GlyphDimensions>,
    font_key: FontKey,
    inst_key: FontInstanceKey,
    size: LayoutSize
}

impl<'a> LayoutedText<'a> {
    fn new(text: &'a str, font_key: FontKey, inst_key: FontInstanceKey, api: &RenderApi) -> Self {
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
            let dx = (g.left + g.width) as f32 + g.advance;
            let dy = (g.top + g.height) as f32;
            (x + dx, y + dy)
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

struct Label<'a> {
    text: LayoutedText<'a>,
    position: LayoutPoint
}

impl<'a> Label<'a> {
    fn new(text: LayoutedText<'a>, position: LayoutPoint) -> Self {
        Label {
            text,
            position
        }
    }
}

impl<'a> Widget for Label<'a> {
    fn draw(&mut self, builder: &mut DisplayListBuilder, space_clip: SpaceAndClipInfo) -> () {

    }
}