use wgpu::{Adapter, Instance};
use winit::{dpi::{LogicalPosition, LogicalSize}, event::{self, Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, monitor::{self, MonitorHandle, VideoMode}, window::{self, Fullscreen, WindowAttributes, WindowBuilder}};
    
async fn run(){
    let window_size = LogicalSize::new(1280, 720);
    
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_inner_size(window_size).with_title("Hello Window uhhh ye").build(&event_loop).unwrap();
    
    window.set_title("Hello WIndow");
    
    event_loop.set_control_flow(ControlFlow::Poll);


    //WGPU stuff

    //Get the default WGPU instance
    let instance = wgpu::Instance::default();
    //Create a WGPU surface to draw on, in this case on the window
    let surface = instance.create_surface(&window).unwrap();
    //Creates an adapter which is something that deals with the GPU
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions{
        compatible_surface: Some(&surface),
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
    }).await.expect("Cant find an adapter");

    //Creates a device which is something that deals with the physical adapter and makes things that can be passed onto command queue, also created here
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
    }, None).await.expect("Couldnt find a adapter for appropriate device");

    // Get the monitor handle for the monitor the windows on
    let monitor_handle = window.current_monitor().unwrap();
    let refresh_rate = monitor_handle.refresh_rate_millihertz().unwrap();

    let monitor_dimensions = monitor_handle.size();

    //window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor_handle))));

    println!("{}, {}", monitor_dimensions.width - window_size.width, monitor_dimensions.height);
    window.set_outer_position(LogicalPosition::new(monitor_dimensions.width/2 - window_size.width/2,
                                                monitor_dimensions.height/2 - window_size.height / 2));

    println!("{}", refresh_rate);

    let window = &window;
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

pub fn init_window(){
    run();
}