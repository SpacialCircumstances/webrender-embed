use webrender::{Renderer, RendererOptions};
use webrender::api::{ColorF, RenderNotifier, GlyphInstance, RenderApi, GlyphOptions, DocumentId, FontInstanceKey, DisplayListBuilder, Transaction, Epoch, PipelineId, CommonItemProperties, SpaceAndClipInfo, PrimitiveFlags, ImageDescriptor, ImageData, ImageFormat, ImageDescriptorFlags, ComplexClipRegion, BorderRadius, ClipMode, BorderStyle, BorderDetails, ImageMask, BorderSide, NormalBorder};
use webrender::api::units::{LayoutSize, DeviceIntSize, LayoutRect, LayoutPoint, LayoutSideOffsets, Au};
use gleam::gl as opengl;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlRequest, Api};
use glutin::dpi::LogicalSize;
use glutin::platform::desktop::EventLoopExtDesktop;
use std::fs::File;
use std::io::Read;
use rusttype::{Font, Scale, Point, PositionedGlyph};
use std::cmp::max;

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

fn render_wr(api: &RenderApi, pipeline_id: PipelineId, txn: &mut Transaction, builder: &mut DisplayListBuilder, font_key: FontInstanceKey, font: &Font) {
    let content_bounds = LayoutRect::new(LayoutPoint::zero(), builder.content_size());
    let root_space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);
    let spatial_id = root_space_and_clip.spatial_id;

    builder.push_simple_stacking_context(
        content_bounds.origin,
        spatial_id,
        PrimitiveFlags::IS_BACKFACE_VISIBLE,
    );

    let clip_id = builder.define_clip(
        &root_space_and_clip,
        content_bounds,
        vec![],
        None
    );

    let text = "Hello World!";
    let layout: Vec<PositionedGlyph> = font.layout(text, Scale::uniform(100.0), Point { x: 0.0, y: 0.0 }).collect();
    let (size_x, size_y) = layout.iter().filter_map(|l| l.pixel_bounding_box()).fold((0, 0), |(x, y), g| (x + g.width(), max(y, g.height())));
    println!("X: {} Y: {}", size_x, size_y);
    let clip_r = LayoutRect::new(LayoutPoint::new(0.0, 0.0), LayoutSize::new(800.0, 600.0));
    let bounds = LayoutRect::new(LayoutPoint::new(0.0, 0.0), LayoutSize::new(size_x as f32, size_y as f32));
    let glyphs: Vec<GlyphInstance> = layout.iter().filter_map(|gl| {
        Some(GlyphInstance {
            index: gl.id().0,
            point: LayoutPoint::new(gl.position().x, gl.position().y + 100.0)
        })
    }).collect();

    builder.push_text(
        &CommonItemProperties::new(
            clip_r,
            SpaceAndClipInfo { spatial_id, clip_id }
        ),
        bounds,
        &glyphs,
        font_key,
        ColorF::new(0.0, 0.0, 1.0, 1.0),
        Some(GlyphOptions::default())
    );

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
    let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
    let mut txn = Transaction::new();

    let font_key = api.generate_font_key();
    let font_inst_key = api.generate_font_instance_key();

    let mut font_file = File::open("OpenSans-Regular.ttf").unwrap();
    let mut font_bytes = Vec::new();
    font_file.read_to_end(&mut font_bytes).unwrap();

    txn.add_raw_font(font_key, font_bytes.clone(), 0);
    txn.add_font_instance(font_inst_key, font_key, Au::new(6000), None, None, vec![]);

    let font = Font::from_bytes(&font_bytes).unwrap();

    render_wr(&api, pipeline_id, &mut txn, &mut builder, font_inst_key, &font);

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
                    },
                    WindowEvent::Resized(size) => {
                        windowed_context.resize(size)
                    },
                    _ => ()
                }
            },
            _ => ()
        }

        renderer.update();
        renderer.render(size).unwrap();
        windowed_context.swap_buffers().ok();
    });

    renderer.deinit();
}