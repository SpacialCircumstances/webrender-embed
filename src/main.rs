use webrender::Renderer;
use gleam::gl;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use glutin::dpi::LogicalSize;

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Embedded webrender")
        .with_inner_size(LogicalSize::new(800, 600));

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

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

            },
            Event::LoopDestroyed => (),
            _ => ()
        }
    });
}