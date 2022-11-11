use std::sync::Arc;
use flux::{core, gfx, math, prefabs, raytrace};

struct RealtimeState {
  // Camera
  camera_yaw: f32,
  camera_pitch: f32,
  camera_distance: f32,
  camera_sensitivity: f32,
  // Raytrace
  render_engine: raytrace::RenderEngine,
  scene_engine: raytrace::SceneEngine,
  // texture: gfx::Texture,
  // screen_quad: core::Node,
}

impl core::AppState for RealtimeState {
  fn init() -> Self {
    let resolution = (400, 400);
    let render_settings = raytrace::RenderSettings {
      resolution,
      ..Default::default()
    };
    let render_engine = raytrace::RenderEngine::new(render_settings);
    let scene_engine = raytrace::SceneEngine::new();

    // let texture = gfx::Texture::new_2d(
    //   resolution,
    //   wgpu::TextureFormat::Rgba8UnormSrgb,
    //   wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    //   1,
    //   1,
    // );
    // let sampler = gfx::Sampler::new(&wgpu::SamplerDescriptor {
    //   address_mode_u: wgpu::AddressMode::ClampToEdge,
    //   address_mode_v: wgpu::AddressMode::ClampToEdge,
    //   address_mode_w: wgpu::AddressMode::ClampToEdge,
    //   mag_filter: wgpu::FilterMode::Linear,
    //   min_filter: wgpu::FilterMode::Nearest,
    //   mipmap_filter: wgpu::FilterMode::Nearest,
    //   ..Default::default()
    // });
    // let texture_bind_group_layout = gfx::BindGroupLayout::new(&wgpu::BindGroupLayoutDescriptor {
    //   label: None,
    //   entries: &[
    //     wgpu::BindGroupLayoutEntry {
    //       binding: 0,
    //       visibility: wgpu::ShaderStages::FRAGMENT,
    //       ty: wgpu::BindingType::Texture {
    //         multisampled: false,
    //         view_dimension: wgpu::TextureViewDimension::D2,
    //         sample_type: wgpu::TextureSampleType::default(),
    //       },
    //       count: None,
    //     },
    //     wgpu::BindGroupLayoutEntry {
    //       binding: 1,
    //       visibility: wgpu::ShaderStages::FRAGMENT,
    //       // This should match the filterable field of the
    //       // corresponding Texture entry above.
    //       ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
    //       count: None,
    //     },
    //   ],
    // });
    // let texture_bind_group = gfx::BindGroup::new(
    //   &texture_bind_group_layout,
    //   &[
    //     gfx::BindGroupEntry::Texture(0, &texture),
    //     gfx::BindGroupEntry::Sampler(1, &sampler),
    //   ],
    // );

    // let material = gfx::MaterialOverlay::new(texture_bind_group);
    // let screen_quad = core::Node::new();
    // let mut mesh = gfx::Mesh::quad();
    // mesh.renderable = false;
    // screen_quad.add_component(mesh);
    // screen_quad.add_component(material);

    Self {
      camera_yaw: 0.0,
      camera_pitch: 0.0,
      camera_distance: 1.0,
      camera_sensitivity: 0.01,
      render_engine,
      scene_engine,
      // texture,
      // screen_quad,
    }
  }
  fn start(&mut self, app: core::AppData) {
    // // Two spheres
    // core::Node::new()
    //   .add_component(gfx::Mesh::sphere(0.5))
    //   .add_component(gfx::Transform::from_translation(glam::Vec3::new(
    //     0.0, 0.0, -1.0,
    //   )));
    // core::Node::new()
    //   .add_component(gfx::Mesh::sphere(100.0))
    //   .add_component(gfx::Transform::from_translation(glam::Vec3::new(
    //     0.0, -100.5, -1.0,
    //   )));
    // let _camera = prefabs::Camera::perspective(90f32.to_radians(), 1.0, 0.01, 1000.0);
  }

  fn update(&mut self, app: core::AppData) {
    // if let Ok(film) = self.render_engine.film.try_read() {
    //   self.texture.update(film.data());
    // }
  }

  fn input(&mut self, app: core::AppData, input: &core::InputSystem) {
    // let world = app.world();
    // let mut start_render = false;
    // {
    //   let mut transform_storage = world.write_storage::<gfx::Transform>();
    //   let camera_storage = world.read_storage::<prefabs::Camera>();

    //   for (transform, camera) in (&mut transform_storage, &camera_storage).join().take(1) {
    //     if input.is_mouse_pressed(winit::event::MouseButton::Right) {
    //       let (dx, dy) = input.mouse_delta();
    //       self.camera_yaw = self.camera_yaw - dx as f32 * self.camera_sensitivity;
    //       self.camera_pitch = self.camera_pitch + dy as f32 * self.camera_sensitivity;
    //     }
    //     if input.scroll_delta().1 != 0.0 {
    //       self.camera_distance = self.camera_distance - input.scroll_delta().1;
    //     }

    //     // calculate view and projection
    //     let view = {
    //       let eye = glam::Vec3::new(
    //         self.camera_yaw.sin() * self.camera_pitch.cos(),
    //         self.camera_pitch.sin(),
    //         self.camera_yaw.cos() * self.camera_pitch.cos(),
    //       ) * self.camera_distance;
    //       let target = glam::Vec3::ZERO;
    //       let up = glam::Vec3::Y;
    //       glam::Mat4::look_at_rh(eye, target, up)
    //     };
    //     *transform = gfx::Transform::from(glam::Affine3A::from_mat4(view));
    //   }

    //   if input.is_key_clicked(winit::event::VirtualKeyCode::Tab) {
    //     let mut mesh_storage = world.write_storage::<gfx::Mesh>();
    //     if let Some(mesh) = mesh_storage.get_mut(self.screen_quad.entity) {
    //       mesh.renderable = !mesh.renderable;
    //       start_render = mesh.renderable;
    //     }
    //   }
    // }
    // if start_render {
    //   self.render_engine.interrupt_render();
    //   self.scene_engine.translate(&app.scene);
    //   let context = self.render_engine.prepare_render(&self.scene_engine);
    //   self.render_engine.render_frame(context);
    // }
  }
}

fn main() {
  let event_loop = winit::event_loop::EventLoop::new();
  let app = flux::core::AppBuilder::new()
    .with_display(&event_loop)
    .with_rendering()
    .build();
  app.run::<RealtimeState>(event_loop);
}
