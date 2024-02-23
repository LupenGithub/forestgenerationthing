use winit::{dpi::{LogicalPosition, LogicalSize}, event::{self, Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, monitor::{self, MonitorHandle, VideoMode}, window::{self, Fullscreen, WindowAttributes, WindowBuilder}};
    
pub fn init_window(){
    let window_size = LogicalSize::new(1280, 720);
    
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_inner_size(window_size).with_title("Hello Window uhhh ye").build(&event_loop).unwrap();
    
    window.set_title("Hello WIndow");
    
    event_loop.set_control_flow(ControlFlow::Poll);

    // Get the monitor handle for the monitor the windows on
    let monitor_handle = window.current_monitor().unwrap();
    let refresh_rate = monitor_handle.refresh_rate_millihertz().unwrap();

    let monitor_dimensions = monitor_handle.size();

    //window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor_handle))));

    println!("{}, {}", monitor_dimensions.width - window_size.width, monitor_dimensions.height);
    window.set_outer_position(LogicalPosition::new(monitor_dimensions.width/2 - window_size.width/2, monitor_dimensions.height/2 - window_size.height / 2));

    println!("{}", refresh_rate);

    event_loop.run(move |event, elwt| {
        match event{
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Stopping the window");
                elwt.exit();
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {event: WindowEvent::RedrawRequested, ..} => {

            }

            _ => ()
        }
    }).unwrap();
}
