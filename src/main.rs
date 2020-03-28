use webrender::{Renderer, RendererOptions};
use webrender::api::*;
use webrender::api::units::{LayoutSize, DeviceIntSize, LayoutRect, LayoutPoint, Au, LayoutVector2D, WorldPoint};
use gleam::gl as opengl;
use glutin::event::{Event, WindowEvent, DeviceEvent, MouseScrollDelta};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlRequest, Api};
use glutin::dpi::LogicalSize;
use glutin::platform::desktop::EventLoopExtDesktop;
use std::fs::File;
use std::io::Read;
use rusttype::{Font, Scale, Point, PositionedGlyph};
use std::cmp::max;
use webrender::euclid::SideOffsets2D;

mod widget;

use widget::*;

struct Notifier<T: 'static + Send> {
    proxy: EventLoopProxy<T>
}

impl<T: 'static + Send> Notifier<T> {
    fn new(el: &EventLoop<T>) -> Self {
        Notifier {
            proxy: el.create_proxy()
        }
    }
}

impl<T: 'static + Send> RenderNotifier for Notifier<T> {
    fn clone(&self) -> Box<dyn RenderNotifier> {
        let notif = Notifier {
            proxy: self.proxy.clone()
        };
        Box::new(notif)
    }

    fn wake_up(&self) {
        println!("Wake up")
    }

    fn new_frame_ready(&self, _: DocumentId, _scrolled: bool, _composite_needed: bool, _render_time_ns: Option<u64>) {
        self.wake_up()
    }
}

pub trait HandyDandyRectBuilder {
    fn to(&self, x2: i32, y2: i32) -> LayoutRect;
    fn by(&self, w: i32, h: i32) -> LayoutRect;
}

// Allows doing `(x, y).to(x2, y2)` or `(x, y).by(width, height)` with i32
// values to build a f32 LayoutRect
impl HandyDandyRectBuilder for (i32, i32) {
    fn to(&self, x2: i32, y2: i32) -> LayoutRect {
        LayoutRect::new(
            LayoutPoint::new(self.0 as f32, self.1 as f32),
            LayoutSize::new((x2 - self.0) as f32, (y2 - self.1) as f32),
        )
    }

    fn by(&self, w: i32, h: i32) -> LayoutRect {
        LayoutRect::new(
            LayoutPoint::new(self.0 as f32, self.1 as f32),
            LayoutSize::new(w as f32, h as f32),
        )
    }
}

fn render_wr(api: &RenderApi, pipeline_id: PipelineId, txn: &mut Transaction, builder: &mut DisplayListBuilder, font_key: FontInstanceKey, font: &Font) {
    let root_space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);
    builder.push_simple_stacking_context(
        LayoutPoint::zero(),
        root_space_and_clip.spatial_id,
        PrimitiveFlags::IS_BACKFACE_VISIBLE,
    );

    // scrolling and clips stuff

    builder.push_simple_stacking_context(
        LayoutPoint::new(10., 10.),
        root_space_and_clip.spatial_id,
        PrimitiveFlags::IS_BACKFACE_VISIBLE,
    );

    // let's make a scrollbox
    let scrollbox = (0, 0).to(300, 400);

    // set the scrolling clip
    let space_and_clip1 = builder.define_scroll_frame(
        &root_space_and_clip,
        None,
        (0, 0).by(1000, 1000),
        scrollbox,
        vec![],
        None,
        ScrollSensitivity::ScriptAndInputEvents,
        LayoutVector2D::zero(),
    );

    // now put some content into it.
    // start with a white background
    let mut info = CommonItemProperties::new((0, 0).to(1000, 1000), space_and_clip1);
    info.hit_info = Some((0, 1));
    builder.push_rect(&info, ColorF::new(1.0, 1.0, 1.0, 1.0));

    // let's make a 50x50 blue square as a visual reference
    let mut info = CommonItemProperties::new((0, 0).to(50, 50), space_and_clip1);
    info.hit_info = Some((0, 2));
    builder.push_rect(&info, ColorF::new(0.0, 0.0, 1.0, 1.0));

    // and a 50x50 green square next to it with an offset clip
    // to see what that looks like
    let mut info = CommonItemProperties::new(
        (50, 0).to(100, 50).intersection(&(60, 10).to(110, 60)).unwrap(),
        space_and_clip1,
    );
    info.hit_info = Some((0, 3));
    builder.push_rect(&info, ColorF::new(0.0, 1.0, 0.0, 1.0));

    let mut info = CommonItemProperties::new(
        (0, 900).to(100, 1000),
        space_and_clip1
    );
    info.hit_info = Some((0, 4));
    builder.push_rect(&info, ColorF::new(1.0, 0.0, 0.0, 1.0));

    builder.pop_stacking_context();
    builder.pop_stacking_context();
}

fn main() {
    let mut el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Embedded webrender")
        .with_inner_size(LogicalSize::new(800, 600));

    let windowed_context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .build_windowed(wb, &el)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let gl = unsafe {
        opengl::GlFns::load_with(
            |symbol| windowed_context.get_proc_address(symbol) as *const _
        )
    };

    let notifier = Notifier::new(&el);
    let options = RendererOptions {
        clear_color: Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
        ..RendererOptions::default()
    };
    let size = DeviceIntSize::new(800, 600);
    let (mut renderer, sender) = Renderer::new(gl.clone(), Box::new(notifier), options, None, size).unwrap();

    let api = sender.create_api();
    let doc_id = api.add_document(size, 0);

    let epoch = Epoch(0);
    let pipeline_id = PipelineId(0, 0);
    let layout_size = size.to_f32() / webrender::euclid::Scale::new(1.0);
    let mut txn = Transaction::new();

    let font_key = api.generate_font_key();
    let font_inst_key = api.generate_font_instance_key();

    let mut font_file = File::open("OpenSans-Regular.ttf").unwrap();
    let mut font_bytes = Vec::new();
    font_file.read_to_end(&mut font_bytes).unwrap();

    txn.add_raw_font(font_key, font_bytes.clone(), 0);
    txn.add_font_instance(font_inst_key, font_key, Au::new(6000), None, None, vec![]);

    api.send_transaction(doc_id, txn);
    renderer.update();

    let mut txn = Transaction::new();
    let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);

    let red = ColorF::new(1.0, 0.0, 0.0, 1.0);

    let label_text = LayoutedText::new("Test!", font_key, font_inst_key, &api);
    let label = Label::new(label_text, LayoutPoint::new(100.0, 100.0), red);
    let mut root = Root::new(Box::new(label));

    let root_space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

    root.draw(&mut builder, root_space_and_clip);

    txn.set_display_list(
        epoch,
        Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
        layout_size,
        builder.finalize(),
        true,
    );
    txn.set_root_pipeline(pipeline_id);
    txn.generate_frame();
    api.send_transaction(doc_id, txn);

    el.run_return(|event, _target, control_flow| {
        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        let mut txn = Transaction::new();

        match event {
            Event::WindowEvent { window_id: _, event } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    }
                    WindowEvent::Resized(size) => {
                        windowed_context.resize(size)
                    }
                    _ => ()
                }
            },
            Event::DeviceEvent { device_id: _, event: DeviceEvent::MouseWheel { delta } } => {
                println!("Scroll: {:#?}", delta);
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => LayoutVector2D::new(x * 20.0, y * 20.0),
                    MouseScrollDelta::PixelDelta(pos) => LayoutVector2D::new(pos.x as f32, pos.y as f32)
                };
                txn.scroll(ScrollLocation::Delta(scroll_delta), WorldPoint::new(100.0, 100.0));
                txn.generate_frame();
            },
            _ => ()
        }

        api.send_transaction(doc_id, txn);

        renderer.update();
        renderer.render(size).unwrap();
        windowed_context.swap_buffers().ok();
    });

    renderer.deinit();
}