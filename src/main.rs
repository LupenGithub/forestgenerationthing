use wgpu::{core::instance, Adapter, Backends, Device, DeviceDescriptor, Features, FragmentState, Instance, InstanceDescriptor, Limits, PipelineLayout, PrimitiveState, Queue, RenderPipeline, RequestAdapterOptions, ShaderModule, Surface, SurfaceCapabilities, SurfaceConfiguration};
use winit::{dpi::{LogicalPosition, LogicalSize}, event::{self, Event, WindowEvent}, event_loop::{self, ControlFlow, EventLoop, EventLoopWindowTarget}, window::{self, Window, WindowAttributes, WindowBuilder}};

struct WindowInfo{
    width: u32,
    height: u32,
    position_x: u32,
    position_y: u32,
    title: &'static str,
    resizable: bool
}

struct Info<'a>{
    event_loop: EventLoopWindowTarget<()>,
    window_info: WindowInfo,
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Surface<'a>,
    window: Window,
    surface_capabilities: SurfaceCapabilities,
    surface_config: SurfaceConfiguration,
    shader_module: ShaderModule,
    render_pipeline_layout: PipelineLayout,
    render_pipeline: RenderPipeline,
}

impl<'a> Info<'a>{
    async fn init_window(&mut self, window_info: WindowInfo){
        env_logger::init();
        
        self.window = WindowBuilder::new().with_inner_size(LogicalSize::new(self.window_info.width, self.window_info.height))
                                               .with_position(LogicalPosition::new(self.window_info.position_x, self.window_info.position_y))
                                               .build(&self.event_loop).unwrap();
        self.instance = Instance::new(InstanceDescriptor{
            backends: Backends::all(),
            ..Default::default()
        });
        self.surface = self.instance.create_surface(&self.window).unwrap();
        self.adapter = self.instance.request_adapter(&RequestAdapterOptions{
            compatible_surface: Some(&self.surface),
            force_fallback_adapter: false,
            power_preference: wgpu::PowerPreference::HighPerformance
        }).await.unwrap();
    }
}


async fn run(){
}

fn main() {
    pollster::block_on(run());
}
