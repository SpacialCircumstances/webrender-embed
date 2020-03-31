use webrender::api::*;
use webrender::api::units::*;
use std::cmp::max;
use crate::text::LayoutedText;

pub struct WebrenderRenderData {
    space_clip: SpaceAndClipInfo
}

impl WebrenderRenderData {
    pub fn new(space_clip: SpaceAndClipInfo) -> Self {
        WebrenderRenderData {
            space_clip
        }
    }
}

pub struct WebrenderUpdateContext<'a> {
    api: &'a RenderApi
}

impl<'a> WebrenderUpdateContext<'a> {
    pub fn new(api: &'a RenderApi) -> Self {
        WebrenderUpdateContext {
            api
        }
    }
}

pub trait Widget {
    fn draw(&mut self, builder: &mut DisplayListBuilder, space_clip: SpaceAndClipInfo) -> ();
}

pub struct Rect {
    area: LayoutRect,
    color: ColorF
}

impl Rect {
    pub fn new(area: LayoutRect, color: ColorF) -> Self {
        Rect {
            area,
            color
        }
    }
}

impl Widget for Rect {
    fn draw(&mut self, builder: &mut DisplayListBuilder, space_clip: SpaceAndClipInfo) -> () {
        let info = CommonItemProperties::new(self.area, space_clip);
        builder.push_rect(&info, self.color);
    }
}

pub struct Root {
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

pub struct Group {
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

pub struct Label<'a> {
    text: LayoutedText<'a>,
    glyph_instances: Vec<GlyphInstance>,
    position: LayoutPoint,
    color: ColorF
}

impl<'a> Label<'a> {
    pub fn new(text: LayoutedText<'a>, position: LayoutPoint, color: ColorF) -> Self {
        let offset = text.dimensions.iter().fold(0.0, |y, &g| {
            let dy = g.height as f32;
            f32::max(y, dy)
        });

        let glyph_instances: Vec<GlyphInstance> = text
            .indices
            .iter()
            .zip(&text.dimensions)
            .scan(position.x, |x, (index, dim)| {
                let tx = *x;
                *x = tx + dim.advance;

                Some(GlyphInstance {
                    index: *index,
                    point: LayoutPoint::new(tx, position.y + offset)
                })
            }).collect();

        Label {
            text,
            position,
            glyph_instances,
            color
        }
    }
}

impl<'a> Widget for Label<'a> {
    fn draw(&mut self, builder: &mut DisplayListBuilder, space_clip: SpaceAndClipInfo) -> () {
        let area = LayoutRect::new(self.position, self.text.size);
        let mut info = CommonItemProperties::new(area, space_clip);
        info.hit_info = Some((0, 1));
        builder.push_text(&info, area, &self.glyph_instances, self.text.inst_key, self.color, Some(GlyphOptions::default()));
    }
}