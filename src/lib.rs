mod gfx;
mod math;
mod rt;
mod core;
use gfx::*;
use winit::{
  dpi::PhysicalSize,
  event::*,
  event_loop::{ControlFlow, EventLoop},
  window::{Window, WindowBuilder},
};

pub trait AppState {
  fn update(&mut self);
  fn resize(&mut self, new_size: &PhysicalSize<u32>);
  fn input(&mut self, event: &WindowEvent) -> bool;
  fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}

pub struct Application {
  event_loop: EventLoop<()>,
  window: Window,
  state: Box<dyn AppState>,
}

impl Application {
  pub fn new() -> Self {
    let size = PhysicalSize::new(400, 400);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_inner_size(size)
      .build(&event_loop)
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
    let state = Box::new(State::new());
    Self {
      event_loop,
      window,
      state,
    }
  }

  pub fn run(mut self) {
    let on_resize = |state: &mut Box<dyn AppState>, new_size: PhysicalSize<u32>| {
      if new_size.width > 0 && new_size.height > 0 {
        context().resize(&new_size);
        state.resize(&new_size);
      }
    };
    let quit = |control_flow: &mut ControlFlow| {
      drop_render_device();
      *control_flow = ControlFlow::Exit;
    };

    self
      .event_loop
      .run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == self.window.id() => {
          self.state.update();
          match self.state.render() {
            Ok(_) => {}
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => on_resize(&mut self.state, self.window.inner_size()),
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => quit(control_flow),
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
          }
        }
        Event::MainEventsCleared => {
          // RedrawRequested will only trigger once, unless we manually
          // request it.
          self.window.request_redraw();
        }
        Event::WindowEvent {
          ref event,
          window_id,
        } if window_id == self.window.id() => {
          if self.state.input(event) {
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
              } => quit(control_flow),
              WindowEvent::Resized(physical_size) => on_resize(&mut self.state, *physical_size),
              WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                on_resize(&mut self.state, **new_inner_size)
              }
              _ => {}
            }
          }
        }
        _ => {}
      });
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

  let app = Application::new();
  app.run();
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

struct State {
  render_pipeline: wgpu::RenderPipeline,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  render_engine: rt::RenderEngine,
  texture: gfx::Texture2D,
  texture_bind_group: wgpu::BindGroup,
}

impl State {
  fn new() -> Self {
    let render_settings = rt::RenderSettings {
      resolution: (400, 400),
      ..Default::default()
    };
    let mut render_engine = rt::RenderEngine::new(render_settings);
    render_engine.render_frame();

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
      texture,
      texture_bind_group,
    }
  }
}

impl AppState for State {
  fn resize(&mut self, new_size: &PhysicalSize<u32>) {}

  fn input(&mut self, event: &WindowEvent) -> bool {
    true
  }

  fn update(&mut self) {}

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
