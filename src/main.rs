use webrender::{Renderer, RendererOptions};
use webrender::api::{ColorF, RenderNotifier, DocumentId, DisplayListBuilder, Transaction, Epoch, PipelineId, CommonItemProperties, SpaceAndClipInfo};
use webrender::api::units::{LayoutSize, DeviceIntSize, LayoutRect, LayoutPoint};
use gleam::gl as opengl;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlRequest, Api};
use glutin::dpi::LogicalSize;
use webrender::euclid::Size2D;

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

fn render_wr(builder: &mut DisplayListBuilder) {
    let rect_size = LayoutRect::new(
        LayoutPoint::new(100.0, 100.0),
        LayoutSize::new(100.0, 200.0)
    );
    let rect_props = CommonItemProperties::new(rect_size, SpaceAndClipInfo::default());
    builder.push_rect(&rect_props, ColorF::new(0.0, 1.0, 1.0, 1.0))
}

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Embedded webrender")
        .with_inner_size(LogicalSize::new(800, 600));

    let windowed_context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .build_windowed(wb, &el)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let gl = unsafe {
        opengl::GlFns::load_with(|ptr| windowed_context.get_proc_address(ptr))
    };

    let notifier = Notifier::new(&el);
    let options = RendererOptions::default();
    let size = DeviceIntSize::new(800, 600);
    let layout_size = LayoutSize::new(800.0, 600.0);
    let (mut renderer, sender) = Renderer::new(gl.clone(), Box::new(notifier), options, None, size).unwrap();

    let api = sender.create_api();
    let doc_id = api.add_document(size, 0);
    let epoch = Epoch(0);
    let pipeline_id = PipelineId(0, 0);
    let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
    let mut txn = Transaction::new();
    render_wr(&mut builder);
    txn.set_display_list(epoch, None, layout_size, builder.finalize(), true);
    txn.generate_frame();
    api.send_transaction(doc_id, txn);

    el.run(move |event, _target, control_flow| {
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
            Event::RedrawRequested(_) => {
                println!("Redraw");
                gl.clear_color(0.5, 0.0, 0.2, 1.0);
                gl.clear(opengl::COLOR_BUFFER_BIT | opengl::DEPTH_BUFFER_BIT | opengl::STENCIL_BUFFER_BIT);
                renderer.update();
                renderer.render(size).unwrap();
                windowed_context.swap_buffers().unwrap();
            },
            Event::LoopDestroyed => (),
            _ => ()
        }
    });

    renderer.deinit();
}