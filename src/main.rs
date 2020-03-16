use webrender::{Renderer, RendererOptions};
use webrender::api::{ColorF, RenderNotifier, RenderApi, DocumentId, DisplayListBuilder, Transaction, Epoch, PipelineId, CommonItemProperties, SpaceAndClipInfo, PrimitiveFlags, ImageDescriptor, ImageData, ImageFormat, ImageDescriptorFlags, ComplexClipRegion, BorderRadius, ClipMode, BorderStyle, BorderDetails, ImageMask, BorderSide, NormalBorder};
use webrender::api::units::{LayoutSize, DeviceIntSize, LayoutRect, LayoutPoint, LayoutSideOffsets};
use gleam::gl as opengl;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlRequest, Api};
use glutin::dpi::LogicalSize;
use webrender::euclid::Size2D;
use glutin::platform::desktop::EventLoopExtDesktop;

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
    fn clone(&self) -> Box<RenderNotifier> {
        let notif = Notifier {
            proxy: self.proxy.clone()
        };
        Box::new(notif)
    }

    fn wake_up(&self) {
        println!("Wake up")
    }

    fn new_frame_ready(&self, _: DocumentId, scrolled: bool, composite_needed: bool, render_time_ns: Option<u64>) {
        self.wake_up()
    }
}

fn render_wr(api: &RenderApi, pipeline_id: PipelineId, txn: &mut Transaction, builder: &mut DisplayListBuilder) {
    let content_bounds = LayoutRect::new(LayoutPoint::zero(), builder.content_size());
    let root_space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);
    let spatial_id = root_space_and_clip.spatial_id;

    builder.push_simple_stacking_context(
        content_bounds.origin,
        spatial_id,
        PrimitiveFlags::IS_BACKFACE_VISIBLE,
    );

    let image_mask_key = api.generate_image_key();
    txn.add_image(
        image_mask_key,
        ImageDescriptor::new(2, 2, ImageFormat::R8, ImageDescriptorFlags::IS_OPAQUE),
        ImageData::new(vec![0, 80, 180, 255]),
        None,
    );
    let mask = ImageMask {
        image: image_mask_key,
        rect: LayoutRect::new(LayoutPoint::new(75.0, 75.0), LayoutSize::new(100.0, 100.0)),
        repeat: false,
    };
    let complex = ComplexClipRegion::new(
        LayoutRect::new(LayoutPoint::new(50.0, 50.0), LayoutSize::new(100.0, 100.0)),
        BorderRadius::uniform(20.0),
        ClipMode::Clip
    );
    let clip_id = builder.define_clip(
        &root_space_and_clip,
        content_bounds,
        vec![complex],
        Some(mask)
    );

    builder.push_rect(
        &CommonItemProperties::new(
            LayoutRect::new(LayoutPoint::new(100.0, 100.0), LayoutSize::new(100.0, 100.0)),
            SpaceAndClipInfo { spatial_id, clip_id },
        ),
        ColorF::new(0.0, 1.0, 0.0, 1.0),
    );

    builder.push_rect(
        &CommonItemProperties::new(
            LayoutRect::new(LayoutPoint::new(250.0, 100.0), LayoutSize::new(100.0, 100.0)),
            SpaceAndClipInfo { spatial_id, clip_id },
        ),
        ColorF::new(0.0, 1.0, 0.0, 1.0),
    );
    let border_side = BorderSide {
        color: ColorF::new(0.0, 0.0, 1.0, 1.0),
        style: BorderStyle::Groove,
    };
    let border_widths = LayoutSideOffsets::new_all_same(10.0);
    let border_details = BorderDetails::Normal(NormalBorder {
        top: border_side,
        right: border_side,
        bottom: border_side,
        left: border_side,
        radius: BorderRadius::uniform(20.0),
        do_aa: true,
    });

    let bounds = LayoutRect::new(LayoutPoint::new(100.0, 100.0), LayoutSize::new(100.0, 100.0));
    builder.push_border(
        &CommonItemProperties::new(
            bounds,
            SpaceAndClipInfo { spatial_id, clip_id },
        ),
        bounds,
        border_widths,
        border_details,
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

    render_wr(&api, pipeline_id, &mut txn, &mut builder);

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
            Event::WindowEvent { window_id, event } => {
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