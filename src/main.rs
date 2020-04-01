use webrender::{Renderer, RendererOptions};
use webrender::api::*;
use webrender::api::units::{LayoutSize, DeviceIntSize, LayoutRect, LayoutPoint, Au, LayoutVector2D, WorldPoint};
use gleam::gl as opengl;
use glutin::event::{Event, WindowEvent, DeviceEvent, MouseScrollDelta, ElementState, MouseButton};
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

mod state;
mod text;
mod component;
mod widget;

use widget::*;
use crate::text::LayoutedText;
use crate::component::Component;
use crate::state::{ImmutableStore, Store};

enum Message {
    Incr
}

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

fn draw_to_transaction<'a, W>(widget: &W, rd: &WebrenderRenderData, pipeline: PipelineId, txn: &mut Transaction, layout_size: LayoutSize, epoch: Epoch) where W: Component<DisplayListBuilder, WebrenderRenderData, WebrenderUpdateContext<'a>, WebrenderEvent> {
    let mut builder = DisplayListBuilder::new(pipeline, layout_size);
    widget.draw(&mut builder, rd);
    txn.generate_frame();
    txn.set_display_list(epoch,
                         Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
                         layout_size,
                         builder.finalize(),
                         true);
    txn.set_root_pipeline(pipeline);

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

    let red = ColorF::new(1.0, 0.0, 0.0, 1.0);
    let green = ColorF::new(0.0, 1.0, 0.0, 1.0);

    let mut state = ImmutableStore::new(0, |&s, m: Message| {
        match m {
            Message::Incr => s + 1
        }
    });

    let mut label = DynamicLabel::new(state.selector(|s| s.to_string()), LayoutPoint::new(0.0, 0.0), red);

    let root_space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);
    let rd = WebrenderRenderData::new(root_space_and_clip);
    let mut uc = WebrenderUpdateContext::new(&api, font_key, font_inst_key);

    label.update(&mut uc);

    let mut txn = Transaction::new();
    draw_to_transaction(&label, &rd, pipeline_id, &mut txn, layout_size, epoch);
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
                    },
                    WindowEvent::Resized(size) => {
                        windowed_context.resize(size)
                    },
                    WindowEvent::MouseInput { device_id: _, state: ElementState::Pressed, button: MouseButton::Left, modifiers: _ } => {
                        state.update(Message::Incr);
                        label.update(&mut uc);
                        draw_to_transaction(&label, &rd, pipeline_id, &mut txn, layout_size, epoch);
                    },
                    WindowEvent::CursorMoved { device_id: _, position, modifiers: _ } => {
                        let point = WorldPoint::new(position.x as f32, position.y as f32);
                        let hit = api.hit_test(doc_id, None, point, HitTestFlags::FIND_ALL);
                        for x in hit.items {
                            println!("Hover over item: ({}, {})", x.tag.0, x.tag.1);
                        }
                    },
                    _ => ()
                }
            },
            Event::DeviceEvent { device_id: _, event } => {
                match event {
                    DeviceEvent::MouseWheel { delta } => {
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
            }
            _ => ()
        }

        api.send_transaction(doc_id, txn);

        renderer.update();
        renderer.render(size).unwrap();
        windowed_context.swap_buffers().ok();
    });

    renderer.deinit();
}