use flux::{core, gfx, math, prefabs, raytrace};

use rand::Rng;
use specs::{Component, DenseVecStorage, Join, WorldExt};
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

// // lib.rs
// const VERTICES: &[Vertex] = &[
//   Vertex {
//     position: [-1.0, -1.0, 0.0],
//     tex_coords: [0.0, 0.0],
//   },
//   Vertex {
//     position: [1.0, -1.0, 0.0],
//     tex_coords: [1.0, 0.0],
//   },
//   Vertex {
//     position: [1.0, 1.0, 0.0],
//     tex_coords: [1.0, 1.0],
//   },
//   Vertex {
//     position: [-1.0, 1.0, 0.0],
//     tex_coords: [0.0, 1.0],
//   },
// ];
// const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];
// const AXIS_HELPER_VERTICES: &[[f32; 6]] = &[
//   [0.0, 0.0, 0.0, 1.0, 0.0, 0.0], // x axis
//   [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
//   [0.0, 0.0, 0.0, 0.0, 1.0, 0.0], // y axis
//   [0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
//   [0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // z axis
//   [0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
// ];

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

  fn update(&mut self, input: &crate::core::InputSystem) {
    if input.is_mouse_pressed(winit::event::MouseButton::Right) {
      let (dx, dy) = input.mouse_delta();
      self.yaw = self.yaw - dx as f32 * self.sensitivity;
      self.pitch = self.pitch + dy as f32 * self.sensitivity;
    }
    if input.scroll_delta().1 != 0.0 {
      self.distance = self.distance - input.scroll_delta().1;
    }
  }
}

struct RealtimeState {
  camera: RealtimeCamera,
  camera_yaw: f32,
  camera_pitch: f32,
  camera_distance: f32,
  camera_sensitivity: f32,
}
impl RealtimeState {
  fn new() -> Self {
    let camera = RealtimeCamera::new(1.0, 60f32.to_radians(), 0.01, 1000.0);
    Self {
      camera,
      camera_yaw: 0.0,
      camera_pitch: 0.0,
      camera_distance: 1.0,
      camera_sensitivity: 0.01,
    }
  }
}

impl core::AppState for RealtimeState {
  fn start(&mut self, app: core::AppData) {
    // Two spheres
    let _top_sphere = prefabs::GeomSphere::new(glam::Vec3::new(0.0, 0.0, -1.0), 0.5);
    let _bottom_sphere = prefabs::GeomSphere::new(glam::Vec3::new(0.0, -100.5, -1.0), 100.0);
    let _camera = prefabs::Camera::perspective(90f32.to_radians(), 1.0, 0.01, 1000.0);
  }

  fn input(&mut self, app: core::AppData, input: &core::InputSystem) {
    let world = app.world();
    let mut transform_storage = world.write_storage::<gfx::Transform>();
    let camera_storage = world.read_storage::<prefabs::Camera>();

    for (transform, camera) in (&mut transform_storage, &camera_storage).join().take(1) {
      if input.is_mouse_pressed(winit::event::MouseButton::Right) {
        let (dx, dy) = input.mouse_delta();
        self.camera_yaw = self.camera_yaw - dx as f32 * self.camera_sensitivity;
        self.camera_pitch = self.camera_pitch + dy as f32 * self.camera_sensitivity;
      }
      if input.scroll_delta().1 != 0.0 {
        self.camera_distance = self.camera_distance - input.scroll_delta().1;
      }

      // calculate view and projection
      let view = {
        let eye = glam::Vec3::new(
          self.camera_yaw.sin() * self.camera_pitch.cos(),
          self.camera_pitch.sin(),
          self.camera_yaw.cos() * self.camera_pitch.cos(),
        ) * self.camera_distance;
        let target = glam::Vec3::ZERO;
        let up = glam::Vec3::Y;
        glam::Mat4::look_at_rh(eye, target, up)
      };
      *transform = gfx::Transform::from(glam::Affine3A::from_mat4(view));
    }
  }
}

// struct RaytraceState {
//   render_pipeline: wgpu::RenderPipeline,
//   vertex_buffer: wgpu::Buffer,
//   index_buffer: wgpu::Buffer,
//   render_engine: raytrace::RenderEngine,
//   scene_engine: Arc<RwLock<raytrace::SceneEngine>>,
//   texture: gfx::Texture2D,
//   texture_bind_group: wgpu::BindGroup,
// }

// impl RaytraceState {
//   fn new() -> Self {
//     let render_settings = raytrace::RenderSettings {
//       resolution: (400, 400),
//       ..Default::default()
//     };
//     let render_engine = raytrace::RenderEngine::new(render_settings);
//     let scene_engine = std::sync::Arc::new(RwLock::new(raytrace::SceneEngine::new()));

//     let shader = gfx::context().create_shader_module(Some("Shader"), include_str!("shader.wgsl"));

//     let texture_bind_group_layout = gfx::context().create_bind_group_layout(
//       Some("texture_bind_group_layout"),
//       &[
//         wgpu::BindGroupLayoutEntry {
//           binding: 0,
//           visibility: wgpu::ShaderStages::FRAGMENT,
//           ty: wgpu::BindingType::Texture {
//             multisampled: false,
//             view_dimension: wgpu::TextureViewDimension::D2,
//             sample_type: wgpu::TextureSampleType::default(),
//           },
//           count: None,
//         },
//         wgpu::BindGroupLayoutEntry {
//           binding: 1,
//           visibility: wgpu::ShaderStages::FRAGMENT,
//           // This should match the filterable field of the
//           // corresponding Texture entry above.
//           ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//           count: None,
//         },
//       ],
//     );

//     let render_pipeline_layout = gfx::context().create_pipeline_layout(
//       Some("Render Pipeline Layout"),
//       &[&texture_bind_group_layout],
//       &[],
//     );

//     let render_pipeline = gfx::context().create_pipeline(
//       Some("Render Pipeline"),
//       Some(&render_pipeline_layout),
//       wgpu::VertexState {
//         module: &shader,
//         entry_point: "vs_main",
//         buffers: &[Vertex::desc()],
//       },
//       Some(wgpu::FragmentState {
//         module: &shader,
//         entry_point: "fs_main",
//         targets: &[Some(wgpu::ColorTargetState {
//           format: gfx::context().surface_format(),
//           blend: Some(wgpu::BlendState::REPLACE),
//           write_mask: wgpu::ColorWrites::ALL,
//         })],
//       }),
//       wgpu::PrimitiveState {
//         topology: wgpu::PrimitiveTopology::TriangleList,
//         strip_index_format: None,
//         front_face: wgpu::FrontFace::Ccw,
//         cull_mode: Some(wgpu::Face::Back),
//         polygon_mode: wgpu::PolygonMode::Fill,
//         unclipped_depth: false,
//         conservative: false,
//       },
//       None,
//       wgpu::MultisampleState {
//         count: 1,
//         mask: !0,
//         alpha_to_coverage_enabled: false,
//       },
//     );

//     let vertex_buffer = gfx::context().create_buffer(
//       Some("Vertex Buffer"),
//       bytemuck::cast_slice(VERTICES),
//       wgpu::BufferUsages::VERTEX,
//     );

//     let index_buffer = gfx::context().create_buffer(
//       Some("Index Buffer"),
//       bytemuck::cast_slice(INDICES),
//       wgpu::BufferUsages::INDEX,
//     );

//     let texture_size = wgpu::Extent3d {
//       width: render_engine.settings.resolution.0,
//       height: render_engine.settings.resolution.1,
//       depth_or_array_layers: 1,
//     };
//     let texture = gfx::Texture2D::new(
//       Some("texture"),
//       texture_size,
//       1,
//       1,
//       wgpu::TextureDimension::D2,
//       wgpu::TextureFormat::Rgba8UnormSrgb,
//       wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//     );
//     let diffuse_sampler = gfx::context().create_sampler(&wgpu::SamplerDescriptor {
//       address_mode_u: wgpu::AddressMode::ClampToEdge,
//       address_mode_v: wgpu::AddressMode::ClampToEdge,
//       address_mode_w: wgpu::AddressMode::ClampToEdge,
//       mag_filter: wgpu::FilterMode::Linear,
//       min_filter: wgpu::FilterMode::Nearest,
//       mipmap_filter: wgpu::FilterMode::Nearest,
//       ..Default::default()
//     });
//     let texture_bind_group = gfx::context().create_bind_group(
//       Some("texture_bind_group"),
//       &texture_bind_group_layout,
//       &[
//         wgpu::BindGroupEntry {
//           binding: 0,
//           resource: wgpu::BindingResource::TextureView(texture.view()),
//         },
//         wgpu::BindGroupEntry {
//           binding: 1,
//           resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
//         },
//       ],
//     );

//     Self {
//       render_pipeline,
//       vertex_buffer,
//       index_buffer,
//       render_engine,
//       scene_engine,
//       texture,
//       texture_bind_group,
//     }
//   }
// }

// impl core::AppState for RaytraceState {
//   fn start(&mut self, app: core::AppData) {
//     self.scene_engine.write().unwrap().translate(&app.scene);
//     let context = self
//       .render_engine
//       .prepare_render(&self.scene_engine.read().unwrap());
//     self.render_engine.render_frame(context);
//   }

//   fn resize(&mut self, new_size: &PhysicalSize<u32>) {}

//   fn render(&mut self, app: core::AppData) -> Result<(), wgpu::SurfaceError> {
//     let output = gfx::context().surface_texture()?;
//     let view = output
//       .texture
//       .create_view(&wgpu::TextureViewDescriptor::default());

//     if let Ok(film) = self.render_engine.film.try_read() {
//       self.texture.update(film.data());
//     }

//     gfx::context().encode_commands(&|encoder: &mut wgpu::CommandEncoder| {
//       let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//         label: Some("Render Pass"),
//         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//           view: &view,
//           resolve_target: None,
//           ops: wgpu::Operations {
//             load: wgpu::LoadOp::Clear(wgpu::Color {
//               r: 0.1,
//               g: 0.2,
//               b: 0.3,
//               a: 1.0,
//             }),
//             store: true,
//           },
//         })],
//         depth_stencil_attachment: None,
//       });
//       render_pass.set_pipeline(&self.render_pipeline);
//       render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
//       render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
//       render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
//       render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
//     });
//     output.present();

//     Ok(())
//   }
// }

fn main() {
  let event_loop = winit::event_loop::EventLoop::new();
  let app = flux::core::AppBuilder::new()
    .with_display(&event_loop)
    .with_rendering()
    .set_initial_state(Box::new(RealtimeState::new()))
    .build();
  app.run(event_loop);
}