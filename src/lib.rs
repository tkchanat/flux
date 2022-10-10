mod core;
pub mod ecs;
mod gfx;
mod math;
mod prefabs;
mod raytrace;

use crate::core::Node;
use gfx::*;
use rand::Rng;
use specs::{Component, DenseVecStorage};
use specs_derive::Component;
use std::{
  io::Empty,
  sync::{Arc, RwLock},
};
use winit::{
  dpi::PhysicalSize,
  event::*,
  event_loop::{ControlFlow, EventLoop},
  window::{Window, WindowBuilder},
};

pub trait AppState {
  fn start(&mut self) {}
  fn update(&mut self, input: &InputSystem) {}
  fn resize(&mut self, new_size: &PhysicalSize<u32>) {}
  fn input(&mut self, input: &InputSystem) -> bool {
    true
  }
  fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    Ok(())
  }
}

static mut APP_INSTANCE: Option<Application> = None;

fn app() -> &'static mut Application {
  unsafe {
    APP_INSTANCE
      .as_mut()
      .expect("Application not initialized yet")
  }
}

pub struct Application {
  quit_requested: bool,
  window: Window,
  input_system: InputSystem,
  state: Box<dyn AppState>,
  scene: core::Scene,
}

impl Application {
  pub fn new(event_loop: &EventLoop<()>) -> Self {
    let size = PhysicalSize::new(400, 400);
    let window = WindowBuilder::new()
      .with_inner_size(size)
      .build(event_loop)
      .unwrap();
    #[cfg(target_arch = "wasm32")]
    {
      // Winit prevents sizing with CSS, so we have to set
      // the size manually when on web.
      window.set_inner_size(size);

      use winit::platform::web::WindowExtWebSys;
      web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
          let dst = doc.get_element_by_id("wasm-example")?;
          let canvas = web_sys::Element::from(window.canvas());
          dst.append_child(&canvas).ok()?;
          Some(())
        })
        .expect("Couldn't append canvas to document body.");
    }

    init_render_device(&window);
    Self {
      quit_requested: false,
      window,
      input_system: InputSystem::new(),
      state: Box::new(RealtimeState::new()),
      scene: core::Scene::new(),
    }
  }
}

impl Application {
  fn event(&mut self, event: &WindowEvent) {
    if app().state.input(&app().input_system) {
      match event {
        WindowEvent::CloseRequested
        | WindowEvent::KeyboardInput {
          input:
            KeyboardInput {
              state: ElementState::Pressed,
              virtual_keycode: Some(VirtualKeyCode::Escape),
              ..
            },
          ..
        } => self.quit(),
        WindowEvent::Resized(physical_size) => self.on_resize(*physical_size),
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.on_resize(**new_inner_size),
        _ => {}
      }
    }
    app().input_system.handle_event(event);
  }
  fn start(&mut self) {
    // Two spheres
    let _top_sphere = prefabs::GeomSphere::new(glam::Vec3::new(0.0, 0.0, -1.0), 0.5);
    let _bottom_sphere = prefabs::GeomSphere::new(glam::Vec3::new(0.0, -100.5, -1.0), 100.0);
    let _camera = prefabs::Camera::perspective(90f32.to_radians(), 1.0, 0.01, 1000.0);
    self.state.start();
  }
  fn update(&mut self) {
    // Input update
    self.input_system.update();

    // Game update
    self.state.update(&self.input_system);

    // Render
    match self.state.render() {
      Ok(_) => {}
      // Reconfigure the surface if lost
      Err(wgpu::SurfaceError::Lost) => self.on_resize(self.window.inner_size()),
      // The system is out of memory, we should probably quit
      Err(wgpu::SurfaceError::OutOfMemory) => self.quit(),
      // All other errors (Outdated, Timeout) should be resolved by the next frame
      Err(e) => eprintln!("{:?}", e),
    }

    // Frame clean up
    self.input_system.scroll_delta = (0.0, 0.0);
  }
  fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      context().resize(&new_size);
      self.state.resize(&new_size);
    }
  }
  fn quit(&mut self) {
    self.quit_requested = true;
  }
}

impl Drop for Application {
  fn drop(&mut self) {
    drop_render_device();
  }
}

pub struct InputSystem {
  mouse_position: Option<(f64, f64)>,
  last_mouse_position: Option<(f64, f64)>,
  mouse_delta: (f64, f64),
  mouse_pressed: [bool; 3],
  scroll_delta: (f32, f32),
  key_pressed: [bool; 163],
}

impl InputSystem {
  pub fn new() -> Self {
    Self {
      mouse_position: None,
      last_mouse_position: None,
      mouse_delta: (0.0, 0.0),
      mouse_pressed: [false; 3],
      scroll_delta: (0.0, 0.0),
      key_pressed: [false; 163],
    }
  }
  pub fn update(&mut self) {
    self.mouse_delta = match self.mouse_position {
      Some(position) => match self.last_mouse_position {
        Some(last_position) => (position.0 - last_position.0, position.1 - last_position.1),
        None => (0.0, 0.0),
      },
      None => (0.0, 0.0),
    };
    self.last_mouse_position = self.mouse_position;
  }
  pub fn handle_event(&mut self, event: &winit::event::WindowEvent) {
    match event {
      WindowEvent::CursorMoved { position, .. } => {
        self.mouse_position = Some((position.x, position.y));
      }
      WindowEvent::CursorLeft { .. } => {
        self.mouse_position = None;
      }
      WindowEvent::MouseInput { state, button, .. } => {
        let pressed = *state == ElementState::Pressed;
        match button {
          MouseButton::Left => {
            self.mouse_pressed[0] = pressed;
          }
          MouseButton::Right => {
            self.mouse_pressed[1] = pressed;
          }
          MouseButton::Middle => {
            self.mouse_pressed[2] = pressed;
          }
          _ => (),
        }
      }
      WindowEvent::MouseWheel { delta, .. } => match delta {
        MouseScrollDelta::LineDelta(dx, dy) => self.scroll_delta = (*dx, *dy),
        MouseScrollDelta::PixelDelta(delta) => {
          let (dx, dy) = (delta.x * 0.1, delta.y * 0.1);
          self.scroll_delta = (dx as f32, dy as f32)
        }
      },
      WindowEvent::KeyboardInput { input, .. } => {
        let pressed = input.state == ElementState::Pressed;
        if let Some(keycode) = input.virtual_keycode {
          self.key_pressed[keycode as usize] = pressed;
        }
      }
      _ => {}
    }
  }
  pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
    let index = match button {
      MouseButton::Left => 0,
      MouseButton::Right => 1,
      MouseButton::Middle => 2,
      MouseButton::Other(_) => panic!("Unsupported mouse button"),
    };
    self.mouse_pressed[index]
  }
  pub fn is_mouse_released(&self, button: MouseButton) -> bool {
    !self.is_mouse_pressed(button)
  }
  pub fn is_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
    self.key_pressed[keycode as usize]
  }
  pub fn is_key_released(&self, keycode: VirtualKeyCode) -> bool {
    !self.is_key_pressed(keycode)
  }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
  cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
      std::panic::set_hook(Box::new(console_error_panic_hook::hook));
      console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    } else {
      env_logger::init();
    }
  }

  let event_loop = EventLoop::new();
  unsafe {
    APP_INSTANCE = Some(Application::new(&event_loop));
  }
  app().start();
  event_loop.run(move |event, _, control_flow| match event {
    // Event::RedrawRequested(window_id) if window_id == self.window.id() => {}
    Event::MainEventsCleared => {
      if app().quit_requested {
        *control_flow = ControlFlow::Exit;
      }
      app().update();
    }
    Event::WindowEvent {
      ref event,
      window_id,
    } if window_id == app().window.id() => {
      if app().state.input(&app().input_system) {
        app().event(&event);
      }
    }
    _ => {}
  });
}

// lib.rs
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
  position: [f32; 3],
  tex_coords: [f32; 2],
}

impl Vertex {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x2,
        },
      ],
    }
  }
}

const VERTICES: &[Vertex] = &[
  Vertex {
    position: [-1.0, -1.0, 0.0],
    tex_coords: [0.0, 0.0],
  },
  Vertex {
    position: [1.0, -1.0, 0.0],
    tex_coords: [1.0, 0.0],
  },
  Vertex {
    position: [1.0, 1.0, 0.0],
    tex_coords: [1.0, 1.0],
  },
  Vertex {
    position: [-1.0, 1.0, 0.0],
    tex_coords: [0.0, 1.0],
  },
];
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];
const AXIS_HELPER_VERTICES: &[[f32; 6]] = &[
  [0.0, 0.0, 0.0, 1.0, 0.0, 0.0], // x axis
  [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
  [0.0, 0.0, 0.0, 0.0, 1.0, 0.0], // y axis
  [0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
  [0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // z axis
  [0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
];

struct RealtimeCamera {
  aspect: f32,
  fov_y: f32,
  near: f32,
  far: f32,
  yaw: f32,
  pitch: f32,
  sensitivity: f32,
  distance: f32,
}

impl RealtimeCamera {
  fn new(aspect: f32, fov_y: f32, near: f32, far: f32) -> Self {
    RealtimeCamera {
      aspect,
      fov_y,
      near,
      far,
      yaw: 0.0,
      pitch: 0.0,
      sensitivity: 0.01,
      distance: 1.0,
    }
  }

  fn view(&self) -> glam::Mat4 {
    let eye = glam::Vec3::new(
      self.yaw.sin() * self.pitch.cos(),
      self.pitch.sin(),
      self.yaw.cos() * self.pitch.cos(),
    ) * self.distance;
    let target = glam::Vec3::ZERO;
    let up = glam::Vec3::Y;
    glam::Mat4::look_at_rh(eye, target, up)
  }

  fn projection(&self) -> glam::Mat4 {
    glam::Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
  }

  fn update(&mut self, input: &InputSystem) {
    if input.is_mouse_pressed(MouseButton::Right) {
      let (dx, dy) = input.mouse_delta;
      self.yaw = self.yaw - dx as f32 * self.sensitivity;
      self.pitch = self.pitch + dy as f32 * self.sensitivity;
    }
    if input.scroll_delta.1 != 0.0 {
      self.distance = self.distance - input.scroll_delta.1;
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
  pub view: [[f32; 4]; 4],
  pub projection: [[f32; 4]; 4],
}

struct RealtimeState {
  pipeline_incandescent: wgpu::RenderPipeline,
  pipeline_axis_helper: wgpu::RenderPipeline,
  axis_helper_buffer: gfx::VertexBuffer,
  sphere: gfx::Mesh,
  camera: RealtimeCamera,
  camera_buffer: gfx::UniformBuffer<CameraUniform>,
  camera_bind_group: wgpu::BindGroup,
  instance_buffer: VertexBuffer,
}

impl RealtimeState {
  fn new() -> Self {
    let axis_helper_buffer = gfx::VertexBuffer::new(bytemuck::cast_slice(AXIS_HELPER_VERTICES));
    let sphere = gfx::Mesh::sphere(10, 10, 1.0);

    let mut rng = rand::thread_rng();
    let positions = (0..1000)
      .map(|x| math::cosine_sample_hemisphere(&glam::Vec2::new(rng.gen(), rng.gen())).to_array())
      .collect::<Vec<_>>();
    let instance_buffer = gfx::VertexBuffer::new(bytemuck::cast_slice(&positions));

    let camera = RealtimeCamera::new(1.0, 60f32.to_radians(), 0.01, 1000.0);
    let camera_uniform = CameraUniform {
      view: camera.view().to_cols_array_2d(),
      projection: camera.projection().to_cols_array_2d(),
    };

    let camera_buffer = context().create_buffer(
      Some("Camera Buffer"),
      bytemuck::cast_slice(&[camera_uniform]),
      wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    );
    let camera_buffer = gfx::UniformBuffer::new();

    let camera_bind_group_layout = context().create_bind_group_layout(
      Some("camera_bind_group_layout"),
      &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    );
    let camera_bind_group = context().create_bind_group(
      Some("camera_bind_group"),
      &camera_bind_group_layout,
      &[wgpu::BindGroupEntry {
        binding: 0,
        resource: camera_buffer.binding(),
      }],
    );

    let render_pipeline_layout = context().create_pipeline_layout(
      Some("Render Pipeline Layout"),
      &[&camera_bind_group_layout],
      &[],
    );
    let shader_incandescent =
      context().create_shader_module(Some("Shader"), include_str!("incandescent.wgsl"));
    let pipeline_incandescent = context().create_pipeline(
      Some("Render Pipeline"),
      Some(&render_pipeline_layout),
      wgpu::VertexState {
        module: &shader_incandescent,
        entry_point: "vs_main",
        buffers: &[
          Vertex::desc(),
          wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
              offset: 0,
              shader_location: 2,
              format: wgpu::VertexFormat::Float32x3,
            }],
          },
        ],
      },
      Some(wgpu::FragmentState {
        module: &shader_incandescent,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: context().surface_format(),
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      None,
      wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
    );

    let shader_axis_helper =
      context().create_shader_module(Some("Shader"), include_str!("axis_helper.wgsl"));
    let pipeline_axis_helper = context().create_pipeline(
      Some("Render Pipeline"),
      Some(&render_pipeline_layout),
      wgpu::VertexState {
        module: &shader_axis_helper,
        entry_point: "vs_main",
        buffers: &[wgpu::VertexBufferLayout {
          array_stride: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes: &[
            wgpu::VertexAttribute {
              offset: 0,
              shader_location: 0,
              format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
              offset: std::mem::size_of::<[f32; 3]>() as u64,
              shader_location: 1,
              format: wgpu::VertexFormat::Float32x3,
            },
          ],
        }],
      },
      Some(wgpu::FragmentState {
        module: &shader_axis_helper,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: context().surface_format(),
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::LineList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      None,
      wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
    );

    Self {
      pipeline_incandescent,
      pipeline_axis_helper,
      axis_helper_buffer,
      sphere,
      camera,
      camera_buffer,
      camera_bind_group,
      instance_buffer,
    }
  }
}
#[derive(Component)]
struct Test(i32);
impl AppState for RealtimeState {
  fn start(&mut self) {
    app().scene.observe::<Test, _>(|node: &Node, comp: &Test| {
      println!("id = {:?}", comp.0);
    });
    let node = Node::new();
    node.add_component(Test(1));
  }

  fn update(&mut self, input: &InputSystem) {
    self.camera.update(&input);
  }

  fn resize(&mut self, new_size: &PhysicalSize<u32>) {}

  fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = context().surface_texture()?;
    let view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    self.camera_buffer.data.view = self.camera.view().to_cols_array_2d();
    self.camera_buffer.data.projection = self.camera.projection().to_cols_array_2d();
    self.camera_buffer.update();

    context().encode_commands(&|encoder: &mut wgpu::CommandEncoder| {
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
            store: true,
          },
        })],
        depth_stencil_attachment: None,
      });

      render_pass.set_pipeline(&self.pipeline_incandescent);
      render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.sphere.vertex_buffer.buffer.slice(..));
      render_pass.set_vertex_buffer(1, self.instance_buffer.buffer.slice(..));
      if let Some(index_buffer) = &self.sphere.index_buffer {
        render_pass.set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..(self.sphere.index_count), 0, 0..1000);
      }

      app().scene.each::<prefabs::Mesh, _>(|mesh| {});

      // // axis helper
      // render_pass.set_pipeline(&self.pipeline_axis_helper);
      // render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
      // render_pass.set_vertex_buffer(0, self.axis_helper_buffer.buffer.slice(..));
      // render_pass.draw(0..AXIS_HELPER_VERTICES.len() as u32, 0..1);
    });
    output.present();

    Ok(())
  }
}

struct RaytraceState {
  render_pipeline: wgpu::RenderPipeline,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  render_engine: raytrace::RenderEngine,
  scene_engine: Arc<RwLock<raytrace::SceneEngine>>,
  texture: gfx::Texture2D,
  texture_bind_group: wgpu::BindGroup,
}

impl RaytraceState {
  fn new() -> Self {
    let render_settings = raytrace::RenderSettings {
      resolution: (400, 400),
      ..Default::default()
    };
    let render_engine = raytrace::RenderEngine::new(render_settings);
    let scene_engine = std::sync::Arc::new(RwLock::new(raytrace::SceneEngine::new()));

    let shader = context().create_shader_module(Some("Shader"), include_str!("shader.wgsl"));

    let texture_bind_group_layout = context().create_bind_group_layout(
      Some("texture_bind_group_layout"),
      &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::default(),
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
    );

    let render_pipeline_layout = context().create_pipeline_layout(
      Some("Render Pipeline Layout"),
      &[&texture_bind_group_layout],
      &[],
    );

    let render_pipeline = context().create_pipeline(
      Some("Render Pipeline"),
      Some(&render_pipeline_layout),
      wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[Vertex::desc()],
      },
      Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: context().surface_format(),
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      None,
      wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
    );

    let vertex_buffer = context().create_buffer(
      Some("Vertex Buffer"),
      bytemuck::cast_slice(VERTICES),
      wgpu::BufferUsages::VERTEX,
    );

    let index_buffer = context().create_buffer(
      Some("Index Buffer"),
      bytemuck::cast_slice(INDICES),
      wgpu::BufferUsages::INDEX,
    );

    let texture_size = wgpu::Extent3d {
      width: render_engine.settings.resolution.0,
      height: render_engine.settings.resolution.1,
      depth_or_array_layers: 1,
    };
    let texture = gfx::Texture2D::new(
      Some("texture"),
      texture_size,
      1,
      1,
      wgpu::TextureDimension::D2,
      wgpu::TextureFormat::Rgba8UnormSrgb,
      wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    let diffuse_sampler = context().create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });
    let texture_bind_group = context().create_bind_group(
      Some("texture_bind_group"),
      &texture_bind_group_layout,
      &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(texture.view()),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
        },
      ],
    );

    Self {
      render_pipeline,
      vertex_buffer,
      index_buffer,
      render_engine,
      scene_engine,
      texture,
      texture_bind_group,
    }
  }
}

impl AppState for RaytraceState {
  fn start(&mut self) {
    self.scene_engine.write().unwrap().translate(&app().scene);
    let context = self
      .render_engine
      .prepare_render(&self.scene_engine.read().unwrap());
    self.render_engine.render_frame(context);
  }

  fn resize(&mut self, new_size: &PhysicalSize<u32>) {}

  fn input(&mut self, input: &InputSystem) -> bool {
    true
  }

  fn update(&mut self, input: &InputSystem) {}

  fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = context().surface_texture()?;
    let view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    if let Ok(film) = self.render_engine.film.try_read() {
      self.texture.update(film.data());
    }

    context().encode_commands(&|encoder: &mut wgpu::CommandEncoder| {
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
            store: true,
          },
        })],
        depth_stencil_attachment: None,
      });
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
      render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
    });
    output.present();

    Ok(())
  }
}
