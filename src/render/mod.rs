use crate::tringulation::Mesh;

pub struct State<'a> {
    // surface: wgpu::Surface<'a>,
    // device: wgpu::Device,
    // queue: wgpu::Queue,
    // config: wgpu::SurfaceConfiguration,
    // size: winit::dpi::PhysicalSize<u32>,
    // // The window must be declared after the surface so
    // // it gets dropped after it as the surface contains
    // // unsafe references to the window's resources.
    window: &'a winit::window::Window,
    // render_pipeline: wgpu::RenderPipeline,
    // vertex_buffer: wgpu::Buffer,
    // index_buffer: wgpu::Buffer,
    test: u8,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a winit::window::Window) -> State<'a> {
        
        Self { test: 69, window }
    }
    // pub fn render_mesh(&mut self, mesh: Mesh, position: Vector3<f32>);
    // pub fn handle_event(&mut self, event: Event) -> bool;
}
