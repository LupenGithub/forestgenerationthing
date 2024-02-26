use wgpu::{Adapter, DeviceDescriptor, Instance, Surface, SurfaceConfiguration};
use winit::{dpi::{LogicalPosition, LogicalSize}, event::{self, Event, WindowEvent}, event_loop::{self, ControlFlow, EventLoop}, monitor::{self, MonitorHandle, VideoMode}, window::{self, Fullscreen, WindowAttributes, WindowBuilder}};
    
struct WindowConfig {
    event_loop: winit::event_loop::EventLoop<()>,
    instance: wgpu::Instance, 
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: winit::window::Window
}

async fn run(mut window_config: WindowConfig) -> WindowConfig{
    window_config.event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event{
            Event::WindowEvent {event: WindowEvent::CloseRequested,.. } =>{
                control_flow.set_exit_with_code(0);
            }, Event::WindowEvent{event: WindowEvent::Resized(new_size),.. } =>{
                if new_size.width > 0 && new_size.height > 0{
                    window_config.size.width = new_size.width;
                    window_config.size.height = new_size.height;
                    window_config.surface.configure(&window_config.device, &window_config.config);
                    println!("{},{}", window_config.size.width, window_config.size.height);
                }
            }, Event::WindowEvent {event: WindowEvent::ScaleFactorChanged {new_inner_size,.. }, ..} => {
                if new_inner_size.width > 0 && new_inner_size.height > 0{
                    window_config.size.width = new_inner_size.width;
                    window_config.size.height = new_inner_size.height;
                    window_config.surface.configure(&window_config.device, &window_config.config);
                    println!("{},{}", window_config.size.width, window_config.size.height);
                }
            },
            _ => ()
        }
    });

    window_config
}

pub async fn init_window(){
    unsafe{    
    let window_size = winit::dpi::PhysicalSize::new(1280, 720);
    
    // Create a new event loop
    let event_loop = winit::event_loop::EventLoop::new();
    // Create a window
    let window = WindowBuilder::new().with_inner_size(window_size).build(&event_loop).unwrap();
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(&window).unwrap(); 

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions{
        compatible_surface : Some(&surface),
        force_fallback_adapter : false,
        power_preference : wgpu::PowerPreference::HighPerformance
    }).await.unwrap();

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
        features: wgpu::Features::empty(),
        label: None,
        limits: wgpu::Limits::default(),
    }, None).await.unwrap();

    let surface_capabilities = surface.get_capabilities(&adapter);

    let surface_caonfiguration = SurfaceConfiguration{
        usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: window_size.width,
        height: window_size.height,
        present_mode: surface_capabilities.present_modes[0],
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: vec![]
    };

    surface.configure(&device, &surface_caonfiguration);
    
    let mut window_config = WindowConfig{event_loop : event_loop, instance : instance, device : device, queue : queue, 
                                                   surface : surface, size : window_size, config : surface_caonfiguration, window};

    pollster::block_on(run(window_config));
    }
}