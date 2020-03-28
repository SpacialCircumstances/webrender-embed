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
    fn new(child: Box<dyn Widget>) -> Self {
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