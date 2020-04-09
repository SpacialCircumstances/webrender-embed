mod state;
mod text;
mod component;
mod widget;

use webrender::{Renderer, RendererOptions};
use webrender::api::*;
use webrender::api::units::{LayoutSize, DeviceIntSize, LayoutRect, LayoutPoint, Au, LayoutVector2D, WorldPoint};
use gleam::gl as opengl;
use glutin::event::{Event, WindowEvent, DeviceEvent, MouseScrollDelta, ElementState, MouseButton};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use glutin::dpi::LogicalSize;
use glutin::platform::desktop::EventLoopExtDesktop;
use std::fs::File;
use std::io::Read;
use widget::*;
use crate::component::Component;
use crate::state::{ImmutableStore, Store};
use luminance_glutin::GlutinSurface;
use luminance::context::GraphicsContext;
use luminance::pipeline::PipelineState;
use luminance_derive::{Semantics, Vertex};
use luminance::render_state::RenderState;
use luminance::tess::{Mode, TessBuilder};
use luminance::shader::{Program, BuiltProgram};
use luminance::tess::SubTess;
use luminance::backend::render_gate::RenderGate;

const VERTEX_SHADER: &str = include_str!("vs.glsl");

const FRAGMENT_SHADER: &str = include_str!("fs.glsl");

#[derive(Copy, Clone, Debug, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "position", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "color", repr = "[u8; 3]", wrapper = "VertexRGB")]
    Color,
}

#[derive(Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    position: VertexPosition,
    #[vertex(normalized = "true")]
    color: VertexRGB,
}

const VERTICES: [Vertex; 3] = [
    Vertex {
        position: VertexPosition::new([-0.5, -0.5]),
        color: VertexRGB::new([255, 0, 0]),
    },
    Vertex {
        position: VertexPosition::new([0.5, -0.5]),
        color: VertexRGB::new([0, 255, 0]),
    },
    Vertex {
        position: VertexPosition::new([0., 0.5]),
        color: VertexRGB::new([0, 0, 255]),
    },
];

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
                         None,
                         layout_size,
                         builder.finalize(),
                         true);
    txn.set_root_pipeline(pipeline);
}

fn main() {
    let (mut surface, mut el) = GlutinSurface::from_builders(
        |win_builder| {
            win_builder
                .with_title("Embedded webrender")
                .with_inner_size(LogicalSize::new(800, 600))
        },
        |ctx_builder| {
            ctx_builder.with_double_buffer(Some(true))
        },
    ).expect("Glutin surface creation");


    let gl = unsafe {
        opengl::GlFns::load_with(
            |symbol| surface.ctx.get_proc_address(symbol) as *const _
        )
    };

    let notifier = Notifier::new(&el);
    let options = RendererOptions {
        clear_color: None,
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

    let image_key = api.generate_image_key();
    let image_file = File::open("planet.png").unwrap();
    let image_reader = BufReader::new(image_file);
    let image = image::load(image_reader, image::ImageFormat::Png)
        .expect("Error loading image");

    let height = image.height();
    let width = image.width();

    let img_and_fmt = match image {
        DynamicImage::ImageLuma8(img) => Ok((img.into_raw(), ImageFormat::R8)),
        DynamicImage::ImageRgba8(img) => Ok((img.into_raw(), ImageFormat::RGBA8)),
        DynamicImage::ImageBgra8(img) => Ok((img.into_raw(), ImageFormat::BGRA8)),
        _ => Err("Unsupported image format")
    };


    let (data, img_fmt) = img_and_fmt.expect("Error decoding image");

    let img_descr = ImageDescriptor::new(width as i32, height as i32, img_fmt, ImageDescriptorFlags::IS_OPAQUE);
    let img_data = ImageData::new(data);
    txn.add_image(image_key, img_descr, img_data, None);

    let font_key = api.generate_font_key();
    let font_inst_key = api.generate_font_instance_key();

    txn.add_native_font(font_key, NativeFontHandle {
        path: PathBuf::from("OpenSans-Regular.ttf"),
        index: 0
    });
    txn.add_font_instance(font_inst_key, font_key, Au::new(6000), None, None, vec![]);

    api.send_transaction(doc_id, txn);
    renderer.update();

    let red = ColorF::new(1.0, 0.0, 0.0, 1.0);
    let blue = ColorF::new(0.0, 0.0, 1.0, 1.0);

    let state = ImmutableStore::new(0, |&s, m: Message| {
        match m {
            Message::Incr => s + 1
        }
    });

    let mut label = DynamicLabel::new(state.selector(|s| s.to_string()), LayoutPoint::new(0.0, 0.0), red);

    let root_space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);
    let rd = WebrenderRenderData::new(root_space_and_clip);
    let mut uc = WebrenderUpdateContext::new(&api, font_key, font_inst_key, image_key);

    label.update(&mut uc);
    //let mut img = ImageDisplay::new(LayoutPoint::new(200.0, 200.0), LayoutSize::new(100.0, 100.0));
    //img.update(&mut uc);

    let mut txn = Transaction::new();
    draw_to_transaction(&label, &rd, pipeline_id, &mut txn, layout_size, epoch);
    api.send_transaction(doc_id, txn);

    let backbuffer = surface.back_buffer().expect("Error loading backbuffer");

    let triangle = TessBuilder::new(&mut surface)
        .unwrap()
        .add_vertices(VERTICES)
        .unwrap()
        .set_mode(Mode::Triangle)
        .unwrap()
        .build()
        .unwrap();

    let program: BuiltProgram<_, VertexSemantics, (), ()> = Program::from_strings(&mut surface, VERTEX_SHADER, None, None, FRAGMENT_SHADER).unwrap();
    for warn in &program.warnings {
        println!("{}", warn);
    }

    let mut program = program.ignore_warnings();

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
                        surface.ctx.resize(size)
                    }
                    WindowEvent::MouseInput { device_id: _, state: ElementState::Pressed, button: MouseButton::Left, modifiers: _ } => {
                        state.update(Message::Incr);
                        label.update(&mut uc);
                        draw_to_transaction(&label, &rd, pipeline_id, &mut txn, layout_size, epoch);
                    }
                    WindowEvent::CursorMoved { device_id: _, position, modifiers: _ } => {
                        let point = WorldPoint::new(position.x as f32, position.y as f32);
                        let hit = api.hit_test(doc_id, None, point, HitTestFlags::FIND_ALL);
                        for x in hit.items {
                            println!("Hover over item: ({}, {})", x.tag.0, x.tag.1);
                        }
                    }
                    _ => ()
                }
            }
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
                    }
                    _ => ()
                }
            }
            _ => ()
        }

        {
            let state_ref = surface.backend().state().clone();
            let mut state = (*state_ref).borrow_mut();
            state.reset_cached_shader_program();
            state.reset_cached_array_buffer();
            state.reset_cached_element_array_buffer();
            state.reset_cached_texture_unit();
            state.reset_cached_vertex_array();
            state.reset_bound_textures();
        }
        
        surface
            .pipeline_gate()
            .pipeline(&backbuffer,
                      &PipelineState::default().set_clear_color(blue.to_array()),
                      |_, mut sh| {
                          sh.shade(&mut program, |_, _, mut rend| {
                              rend.render(&RenderState::default(), |mut tess| {
                                  tess.render(triangle.slice(..).unwrap())
                              })
                          })
                      });

        api.send_transaction(doc_id, txn);

        renderer.update();
        renderer.render(size).unwrap();

        surface.swap_buffers();
    });

    renderer.deinit();
}