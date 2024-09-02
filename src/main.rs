use camera::{Camera, CameraController, CameraUniform, Projection};
use cgmath::{Point3, Rad};
use noise::{NoiseFn, Perlin};
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::PhysicalKey,
    window::{Window, WindowBuilder},
};

mod camera;
mod texture;
mod vertex;
use texture::Texture;
use vertex::Vertex;

struct Chunk {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    position_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pos: [f32; 3],
}

impl Chunk {
    fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("position_bind_group_layout"),
        })
    }

    fn new(
        heightmap: &Vec<Vec<(f32, [f32; 3])>>,
        pos: [f32; 3],
        device: &wgpu::Device,
        mut n_vertices: Vec<Vertex>,
        n_indices: Vec<u32>,
    ) -> Self {
        let now = std::time::Instant::now();
        let (mut vertices, mut indices) = create_mesh(&heightmap);
        let next_index = vertices.len() as u32;
        for n in n_vertices {
            vertices.push(n);
        }
        for n in n_indices {
            indices.push(n + next_index);
        }
        // indices.extend(n_indices.iter().map(|index| index + next_index));
        // vertices = n_vertices;
        // indices = n_indices;
        println!("meshing time: {:?}", now.elapsed());
        let now = std::time::Instant::now();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Position Buffer"),
            contents: bytemuck::cast_slice(&[pos]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let position_bind_group_layout = Self::get_bind_group_layout(device);

        let position_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &position_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: position_buffer.as_entire_binding(),
            }],
            label: Some("position_bind_group"),
        });
        println!("gpu-buffering time: {:?}", now.elapsed());

        Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            position_buffer,
            bind_group: position_bind_group,
            pos,
        }
    }
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    chunks: Vec<Chunk>,
    diffuse_bind_group: wgpu::BindGroup,

    camera: Camera,
    projection: Projection,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    mouse_pressed: bool,
    depth_texture: Texture,
}

async fn make_adapter<'a>(instance: &wgpu::Instance, surface: &wgpu::Surface<'a>) -> wgpu::Adapter {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap()
}

fn get_surface_format(surface_caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
    // get an sRGB format, if available, else the first supported format
    surface_caps
        .formats
        .iter()
        .copied()
        .filter(|f| f.is_srgb())
        .next()
        .unwrap_or(surface_caps.formats[0])
}

fn make_surface_config(
    surface_caps: &wgpu::SurfaceCapabilities,
    size: &PhysicalSize<u32>,
) -> wgpu::SurfaceConfiguration {
    let surface_format = get_surface_format(&surface_caps);
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 1,
    }
}

fn make_render_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    vertex_buffer_layout: wgpu::VertexBufferLayout,
    config: &wgpu::SurfaceConfiguration,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex_buffer_layout],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, //Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window, so this should be safe.
        let surface = instance.create_surface(window).unwrap();

        let adapter = make_adapter(&instance, &surface).await;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let config = make_surface_config(&surface_caps, &size);
        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse =
            Texture::from_bytes(&device, &queue, diffuse_bytes, Some("happy-tree.png")).unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera = Camera::new(Point3::new(0.0, 2.0, 1.0), Rad(0.0), Rad(0.0));
        let projection = Projection::new(
            /*width:*/ config.width,
            /*height:*/ config.height,
            /*fovy:*/ Rad(0.79),
            /*znear:*/ 0.1,
            /*zfar:*/ 100.0,
        );

        let camera_controller = CameraController::new(10.0, 4.0);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let vertex_buffer_layout = Vertex::get_layout();
        let render_pipeline = make_render_pipeline(
            &device,
            &shader,
            vertex_buffer_layout,
            &config,
            &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
                &Chunk::get_bind_group_layout(&device),
            ],
        );

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let mut chunks = Vec::new();

        for cx in 0..1 {
            for cy in 0..1 {
                let x = cx as f32 * 100.0;
                let y = cy as f32 * 100.0;
                let (heightmap, vertices, indices) = create_heightmap([x, 0.0, y]);
                let chunk = Chunk::new(&heightmap, [x, 0.0, y], &device, vertices, indices);
                chunks.push(chunk);
            }
        }

        Self {
            window: &window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            diffuse_bind_group,
            camera,
            camera_buffer,
            projection,
            camera_bind_group,
            camera_controller,
            camera_uniform,
            mouse_pressed: false,
            chunks,
            depth_texture,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.projection.resize(self.size.width, self.size.height);
            // TODO update uniform
        }
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            for chunk in &self.chunks {
                render_pass.set_vertex_buffer(0, chunk.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(chunk.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_bind_group(2, &chunk.bind_group, &[]);
                render_pass.draw_indexed(0..chunk.indices.len() as u32, 0, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
fn noise_fract(x: f32, y: f32, octaves: usize, gen: &Perlin) -> f32 {
    let mut acc = 0.0;
    let lacunarity = 2.0;
    let gain = 0.4;
    let mut amplitude = 0.5;
    let mut frequency = 1.;
    for i in 1..octaves {
        // acc = acc
        //     + gen.get([
        //         x / (2.0f32.powf(i as f32)) + i as f32 * 1000.0,
        //         y / (2.0f32.powf(i as f32)) + i as f32 * 1000.0,
        //     ]) * i as f32;

        acc += amplitude
            * gen.get([
                (frequency * x + i as f32 * 1000.0) as f64,
                (frequency * y + i as f32 * 1000.0) as f64,
            ]) as f32;
        frequency *= lacunarity;
        amplitude *= gain;
    }
    return acc;
}

fn vec_addeq(a: &mut [f32], b: &[f32]) {
    a[0] += b[0];
    a[1] += b[1];
    a[2] += b[2];
}

fn sample_bilinear(heightmap: &[Vec<(f32, [f32; 3])>], pos: [f32; 2]) -> (f32, [f32; 2]) {
    let (x, y) = (pos[0], pos[1]);
    let dx = x - x.floor();
    let dy = y - y.floor();

    let (hx, hy) = (x.floor() as usize, y.floor() as usize);

    let h00 = heightmap[hy][hx].0;
    let h01 = heightmap[hy + 1][hx].0;
    let h10 = heightmap[hy][hx + 1].0;
    let h11 = heightmap[hy + 1][hx + 1].0;

    let mut grad = [
        // (h01 - h00) * (1.0 - dy) + (h11 - h10) * dy,
        // (h10 - h00) * (1.0 - dx) + (h11 - h01) * dx,
        (h10 - h00) * (1.0 - dy) + (h11 - h01) * dy,
        (h01 - h00) * (1.0 - dx) + (h11 - h10) * dx,
    ];
    // let inv_len = 1.0 / (grad[0].powi(2) + grad[1].powi(2)).sqrt();
    // grad[0] *= inv_len;
    // grad[1] *= inv_len;

    let sample = h00 * (1.0 - dx) * (1.0 - dy)
        + h01 * dy * (1.0 - dx)
        + h10 * (1.0 - dy) * dx
        + h11 * dx * dy;
    (sample, grad)
}

fn add_bilinear(heightmap: &mut [Vec<(f32, [f32; 3])>], pos: [f32; 2], amt: f32) {
    let (x, y) = (pos[0], pos[1]);
    let dx = x - x.floor();
    let dy = y - y.floor();

    let (hx, hy) = (x.floor() as usize, y.floor() as usize);
    heightmap[hy][hx].0 += amt * (1.0 - dx) * (1.0 - dy);
    heightmap[hy + 1][hx].0 += amt * dy * (1.0 - dx);
    heightmap[hy][hx + 1].0 += amt * (1.0 - dy) * dx;
    heightmap[hy + 1][hx + 1].0 += amt * dx * dy;
}

fn create_heightmap(pos: [f32; 3]) -> (Vec<Vec<(f32, [f32; 3])>>, Vec<Vertex>, Vec<u32>) {
    let now = std::time::Instant::now();
    let noise = Perlin::new(1);
    let rad = 100;
    let mut heightmap = (0..rad)
        .map(|y| {
            (0..rad)
                .map(|x| {
                    (
                        noise_fract(
                            (x as f32 + pos[0]) / 168.0,
                            (y as f32 + pos[2]) / 168.0,
                            10,
                            &noise,
                        ) as f32
                            / 0.01,
                        [0.0, 0.0, 0.0],
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    println!("Heightmap creation: {:?}", now.elapsed());
    let now = std::time::Instant::now();
    for y in 0..heightmap.len() - 1 {
        for x in 0..heightmap[y].len() - 1 {
            let (height, grad) = sample_bilinear(&heightmap, [x as f32, y as f32]);
            let new_pos = [x as f32 + grad[0], y as f32 + grad[0]];
            let asdf = if new_pos[0] >= (rad - 1) as f32
                || new_pos[1] >= (rad - 1) as f32
                || new_pos[0] <= 0.0
                || new_pos[1] <= 0.0
            {
                0.0
            } else {
                let (nh, _) = sample_bilinear(&heightmap, new_pos);
                nh - height
            };
            heightmap[y][x].1 = [grad[0], asdf, grad[1]];
        }
    }

    let mut rng = rand::thread_rng();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for _ in 0..5000 {
        let mut drop_pos = [
            rng.gen::<f32>() * (rad - 1) as f32,
            rng.gen::<f32>() * (rad - 1) as f32,
        ];
        let mut water_volume = 1.0;
        let mut sediment_volume = 0.0;
        let mut speed = 1.0;

        let mut started_drawing = false;
        while water_volume > 0.3 {
            let (height, mut gradient) = sample_bilinear(&heightmap, drop_pos);
            let len = gradient[0].powi(2) + gradient[1].powi(2);
            if len != 0.0 {
                let inv_len = 1.0 / len;
                gradient[0] *= inv_len;
                gradient[1] *= inv_len;
            }
            drop_pos[0] -= gradient[0];
            drop_pos[1] -= gradient[1];
            if drop_pos[0] >= (rad - 1) as f32
                || drop_pos[1] >= (rad - 1) as f32
                || drop_pos[0] <= 0.0
                || drop_pos[1] <= 0.0
            {
                break;
            }

            let (new_height, _) = sample_bilinear(&heightmap, drop_pos);
            let delta_height = height - new_height;

            let sediment_capacity = f32::max(delta_height * speed * water_volume * 4.0, 0.01);
            if sediment_volume > sediment_capacity {
                // deposit
                let deposit_quantity = if delta_height > 0.0 {
                    f32::min(sediment_volume, delta_height)
                } else {
                    (sediment_volume - sediment_capacity) * 0.3
                };

                add_bilinear(&mut heightmap, drop_pos, deposit_quantity);
                sediment_volume -= deposit_quantity;
            } else {
                // erode
                // delta height is likely negative (I think?)
                let erode_quantity =
                    f32::min((sediment_capacity - sediment_volume) * 0.3, -delta_height);
                sediment_volume += erode_quantity;
                add_bilinear(&mut heightmap, drop_pos, -erode_quantity);
            }

            if started_drawing {
                let last = vertices.len() as u32 - 2;
                indices.push(last);
                indices.push(last + 1);
                indices.push(last + 2);
                indices.push(last + 2);
                indices.push(last + 3);
                indices.push(last + 1);
            }
            let normal = [0.0, sediment_capacity, 0.0];
            // started_drawing = true;
            vertices.push(Vertex {
                position: [drop_pos[0], height + 1.0, drop_pos[1]],
                normal,
            });
            vertices.push(Vertex {
                position: [drop_pos[0] + 0.5, height + 1.0, drop_pos[1] + 0.5],
                normal,
            });
            water_volume *= 0.97; // 0.99
            speed = f32::sqrt(speed * speed + delta_height * 4.0);
        }
    }
    println!("Erosion: {:?}", now.elapsed());

    (heightmap, vertices, indices)
}

fn create_mesh(heightmap: &[Vec<(f32, [f32; 3])>]) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut n_vertices = Vec::new();
    let mut n_indices: Vec<u32> = Vec::new();
    let width = heightmap[0].len();
    let height = heightmap.len();
    for y in 0..height {
        for x in 0..width {
            vertices.push(Vertex {
                position: [x as f32, heightmap[y][x].0, y as f32],
                normal: heightmap[y][x].1,
            });

            // skip last point in each row or col as it has already been included in a triangle
            if x == height - 1 || y == width - 1 {
                continue;
            }
            // quad: 6 vertices
            // first tri
            indices.push(((y + 1) * width + x + 1) as u32);
            indices.push((y * width + x + 1) as u32);
            indices.push((y * width + x) as u32);
            // second tri
            indices.push(((y + 1) * width + x) as u32);
            indices.push(((y + 1) * width + x + 1) as u32);
            indices.push((y * width + x) as u32);

            // normal visualisation
            n_vertices.push(Vertex {
                position: [x as f32, heightmap[y][x].0, y as f32],
                normal: [0.0, 0.0, 0.0],
            });
            n_vertices.push(Vertex {
                position: [x as f32, heightmap[y][x].0, y as f32 + 0.2],
                normal: [1.0, 0.0, 0.0],
            });
            n_vertices.push(Vertex {
                position: [
                    x as f32 + heightmap[y][x].1[0] * 3.0,
                    heightmap[y][x].0 + heightmap[y][x].1[1] * 3.0,
                    y as f32 + heightmap[y][x].1[2] * 3.0,
                ],
                normal: [0.0, 1.0, 0.0],
            });
            n_vertices.push(Vertex {
                position: [
                    x as f32 + heightmap[y][x].1[0] * 3.0,
                    heightmap[y][x].0 + heightmap[y][x].1[1] * 3.0,
                    y as f32 + heightmap[y][x].1[2] * 3.0 + 0.2,
                ],
                normal: [0.0, 0.0, 1.0],
            });

            let l = n_vertices.len() as u32 - 4;
            n_indices.extend([l, l + 1, l + 2, l + 1, l + 2, l + 3].iter());
        }
    }

    let next_index = vertices.len();
    // vertices.append(&mut n_vertices);
    // indices.extend(n_indices.iter().map(|index| index + next_index as u32));
    // for tri in indices.chunks(3) {
    //     let (a, b, c) = (tri[0], tri[1], tri[2]);
    //     let va = &vertices[a as usize].position;
    //     let vb = &vertices[b as usize].position;
    //     let vc = &vertices[c as usize].position;
    //     let (a1, a2, a3) = (vb[0] - va[0], vb[1] - va[1], vb[2] - va[1]);
    //     let (b1, b2, b3) = (vc[0] - va[0], vc[1] - va[1], vc[2] - va[1]);
    //     // cross product
    //     let face_normal = [a2 * b3 - a3 * b2, a3 * b1 - a1 * b3, a1 * b2 - a2 * b1];
    //     vec_addeq(&mut vertices[a as usize].normal, &face_normal);
    //     vec_addeq(&mut vertices[b as usize].normal, &face_normal);
    //     vec_addeq(&mut vertices[c as usize].normal, &face_normal);
    // }

    // for vertex in &mut vertices {
    //     let inv_len = 1.0
    //         / (vertex.normal[0].powi(2) + vertex.normal[1].powi(2) + vertex.normal[2].powi(2))
    //             .sqrt();
    //     vertex.normal[0] *= inv_len;
    //     vertex.normal[1] *= inv_len;
    //     vertex.normal[2] *= inv_len;
    // }

    (vertices, indices)
}

async fn run() {
    // let mut map: Vec<Vec<f32>> = vec![
    //     vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    //     vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    //     vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
    //     vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
    //     vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    //     vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    // ];
    // add_bilinear(&mut map, [0.5, 0.5], 1.0);
    // let chars = [' ', '.', ':', '*', 'O', '%', '#'];

    // let scl = 1;
    // for y in 0..5 * scl {
    //     for x in 0..5 * scl {
    //         let (nx, ny) = (x as f32 / scl as f32, y as f32 / scl as f32);
    //         // let n = map[y][x];
    //         // println!("{nx},{ny}");
    //         let n = sample_bilinear(&map, [nx, ny]);
    //         // println!("{n}");
    //         print!("{}", chars[(n * (chars.len() - 1) as f32).floor() as usize]);
    //         print!("{}", chars[(n * (chars.len() - 1) as f32).floor() as usize]);
    //     }
    //     println!();
    // }

    let event_loop = EventLoop::new().expect("Unable to create event loop!");
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;
    let mut last_render_time = std::time::Instant::now();
    let mut last_print_time = std::time::Instant::now();
    event_loop
        .run(move |event, elwt| match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => state.camera_controller.process_mouse(delta.0, delta.1),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.window.request_redraw();
                            let now = std::time::Instant::now();
                            let dt = now - last_render_time;
                            last_render_time = now;
                            if last_print_time.elapsed().as_millis() > 500 {
                                last_print_time = now;
                                println!("{:?}", dt);
                            }
                            state.update(dt);
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Event::MainEventsCleared => {
            //     // RedrawRequested will only trigger once unless we manually
            //     // request it.
            //     state.window.request_redraw();
            // }
            _ => {}
        })
        .unwrap();
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
