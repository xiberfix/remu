use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("remu")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)
        .expect("failed to create window");

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id } => {
                if window.id() == window_id {
                    control_flow.set_exit()
                }
            }
            _ => ()
        }
    });
}
