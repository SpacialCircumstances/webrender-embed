use webrender::Renderer;
use gleam::gl as opengl;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlRequest, Api};
use glutin::dpi::LogicalSize;

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
                windowed_context.swap_buffers();
            },
            Event::LoopDestroyed => (),
            _ => ()
        }
    });
}