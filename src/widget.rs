use webrender::api::*;
use webrender::api::units::*;
use std::cmp::max;
use crate::text::LayoutedText;
use crate::component::Component;

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

pub enum WebrenderEvent {

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

impl<'a> Component<DisplayListBuilder, WebrenderRenderData, WebrenderUpdateContext<'a>, WebrenderEvent> for Rect {
    fn draw(&self, ctx: &mut DisplayListBuilder, render_data: &WebrenderRenderData) {
        let info = CommonItemProperties::new(self.area, render_data.space_clip);
        ctx.push_rect(&info, self.color);
    }

    fn update(&mut self, _: &mut WebrenderUpdateContext<'a>) {

    }

    fn handle_event(&mut self, _: WebrenderEvent) {

    }
}

pub struct StaticLabel<'a> {
    text: LayoutedText<'a>,
    glyph_instances: Vec<GlyphInstance>,
    position: LayoutPoint,
    color: ColorF
}

impl<'a> StaticLabel<'a> {
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

        StaticLabel {
            text,
            position,
            glyph_instances,
            color
        }
    }
}

impl<'a> Component<DisplayListBuilder, WebrenderRenderData, WebrenderUpdateContext<'a>, WebrenderEvent> for StaticLabel<'a> {
    fn draw(&self, ctx: &mut DisplayListBuilder, render_data: &WebrenderRenderData) {
        let area = LayoutRect::new(self.position, self.text.size);
        let mut info = CommonItemProperties::new(area, render_data.space_clip);
        info.hit_info = Some((0, 1));
        ctx.push_text(&info, area, &self.glyph_instances, self.text.inst_key, self.color, Some(GlyphOptions::default()));
    }

    fn update(&mut self, ctx: &mut WebrenderUpdateContext<'a>) {
    }

    fn handle_event(&mut self, event: WebrenderEvent) {
        //TODO
    }
}