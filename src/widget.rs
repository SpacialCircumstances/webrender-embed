use webrender::api::*;
use webrender::api::units::*;
use std::cmp::max;
use crate::text::LayoutedText;
use crate::component::Component;
use crate::state::Selector;

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
    api: &'a RenderApi,
    font: FontKey,
    font_inst: FontInstanceKey,
    img: ImageKey
}

impl<'a> WebrenderUpdateContext<'a> {
    pub fn new(api: &'a RenderApi, font: FontKey, font_inst: FontInstanceKey, img: ImageKey) -> Self {
        WebrenderUpdateContext {
            api,
            font,
            font_inst,
            img
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

pub struct StaticLabel {
    text: LayoutedText,
    glyph_instances: Vec<GlyphInstance>,
    position: LayoutPoint,
    color: ColorF
}

impl StaticLabel {
    pub fn new(text: LayoutedText, position: LayoutPoint, color: ColorF) -> Self {
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

impl Component<DisplayListBuilder, WebrenderRenderData, WebrenderUpdateContext<'_>, WebrenderEvent> for StaticLabel {
    fn draw(&self, ctx: &mut DisplayListBuilder, render_data: &WebrenderRenderData) {
        let area = LayoutRect::new(self.position, self.text.size);
        let mut info = CommonItemProperties::new(area, render_data.space_clip);
        info.hit_info = Some((0, 1));
        ctx.push_text(&info, area, &self.glyph_instances, self.text.inst_key, self.color, Some(GlyphOptions::default()));
    }

    fn update(&mut self, ctx: &mut WebrenderUpdateContext<'_>) {
    }

    fn handle_event(&mut self, event: WebrenderEvent) {
        //TODO
    }
}

pub struct DynamicLabel<'a, S> where S: Into<String> {
    text_selector: Selector<'a, S>,
    text: Option<LayoutedText>,
    glyph_instances: Vec<GlyphInstance>,
    position: LayoutPoint,
    color: ColorF
}

impl<'a, S> DynamicLabel<'a, S> where S: Into<String> {
    pub fn new(text_selector: Selector<'a, S>, position: LayoutPoint, color: ColorF) -> Self {
        DynamicLabel {
            text_selector,
            position,
            glyph_instances: Vec::new(),
            text: None,
            color
        }
    }
}

impl<'a, 'b, S> Component<DisplayListBuilder, WebrenderRenderData, WebrenderUpdateContext<'b>, WebrenderEvent> for DynamicLabel<'a, S> where S: Into<String> {
    fn draw(&self, ctx: &mut DisplayListBuilder, render_data: &WebrenderRenderData) {
        let text = self.text.as_ref().unwrap();
        let area = LayoutRect::new(self.position, text.size);
        let mut info = CommonItemProperties::new(area, render_data.space_clip);
        info.hit_info = Some((0, 1));
        ctx.push_text(&info, area, &self.glyph_instances, text.inst_key, self.color, Some(GlyphOptions::default()));
    }

    fn update(&mut self, ctx: &mut WebrenderUpdateContext<'b>) {
        let new_text = (self.text_selector)().into();

        if let Some(old_text) = &self.text {
            if old_text.text == new_text {
                return
            }
        }

        let text = LayoutedText::new(new_text, ctx.font, ctx.font_inst, ctx.api);

        let offset = text.dimensions.iter().fold(0.0, |y, &g| {
            let dy = g.height as f32;
            f32::max(y, dy)
        });

        self.glyph_instances = text
            .indices
            .iter()
            .zip(&text.dimensions)
            .scan(self.position.x, |x, (index, dim)| {
                let tx = *x;
                *x = tx + dim.advance;

                Some(GlyphInstance {
                    index: *index,
                    point: LayoutPoint::new(tx, self.position.y + offset)
                })
            }).collect();

        self.text = Some(text);
    }

    fn handle_event(&mut self, event: WebrenderEvent) {
    }
}