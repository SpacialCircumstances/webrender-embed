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
    indices: Vec<Option<u32>>,
    dimensions: Vec<Option<GlyphDimensions>>,
    font_key: FontKey,
    inst_key: FontInstanceKey
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
        //TODO: Draw
    }
}