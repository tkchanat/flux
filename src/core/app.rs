use std::{cell::Ref, collections::VecDeque, fmt::Display, sync::Arc};

use super::{input::InputSystem, Scene};
use crate::gfx::{self, Renderer};
use flux_gfx::device::RenderDevice;
use winit::{
  dpi::PhysicalSize,
  event::*,
  event_loop::{ControlFlow, EventLoop},
  window::{Window, WindowBuilder},
};

pub trait AppState {
  fn init() -> Self
  where
    Self: Sized;
  fn start(&mut self, app: AppData) {}
  fn update(&mut self, app: AppData) {}
  fn resize(&mut self, new_size: &PhysicalSize<u32>) {}
  fn input(&mut self, app: AppData, input: &InputSystem) {}
}

pub struct AppData<'a> {
  window: &'a winit::window::Window,
  pub scene: &'a Scene,
}
impl<'a> AppData<'a> {
  pub fn window_width(&self) -> u32 {
    self.window.inner_size().width
  }
  pub fn window_height(&self) -> u32 {
    self.window.inner_size().height
  }
}

pub(crate) static mut APP_INSTANCE: Option<Application> = None;

pub(super) fn app() -> &'static mut Application {
  unsafe {
    APP_INSTANCE
      .as_mut()
      .expect("Application not initialized yet")
  }
}

struct EmptyState;
impl AppState for EmptyState {
  fn init() -> Self
  where
    Self: Sized,
  {
    Self {}
  }
}
pub enum Transition {
  None,
  Pop,
  Push(Box<dyn AppState>),
  Switch(Box<dyn AppState>),
  Quit,
}

struct DisplaySystem {
  window: Arc<winit::window::Window>,
}
struct RenderingSystem {
  device: Arc<RenderDevice>,
  renderer: Box<dyn gfx::Renderer>,
}

#[derive(Default)]
pub struct AppBuilder {
  display_system: Option<DisplaySystem>,
  rendering_system: Option<RenderingSystem>,
}
impl AppBuilder {
  pub fn new() -> Self {
    Self::default()
  }
  pub fn with_display(mut self, event_loop: &EventLoop<()>) -> AppBuilder {
    let size = PhysicalSize::new(400, 400);
    let window = Arc::new(
      WindowBuilder::new()
        .with_inner_size(size)
        .build(event_loop)
        .unwrap(),
    );
    self.display_system = Some(DisplaySystem { window });
    self
  }
  pub fn with_rendering(mut self) -> AppBuilder {
    let window = self
      .display_system
      .as_ref()
      .expect("Rendering system depends on display system")
      .window
      .clone();
    let mut device = RenderDevice::new(Some(window));
    let renderer = Box::new(gfx::StandardRenderer::new(&mut device));
    self.rendering_system = Some(RenderingSystem { device, renderer });
    self
  }
  pub fn build(self) -> Application {
    let display = self.display_system.unwrap();
    let rendering = self.rendering_system.unwrap();
    Application {
      scene: Scene::new(),
      window: display.window,
      render_device: rendering.device,
      renderer: rendering.renderer,
      input_system: InputSystem::new(),
      states: VecDeque::new(),
      quit_requested: false,
    }
  }
}

pub struct Application {
  pub(crate) render_device: Arc<RenderDevice>,
  pub(crate) scene: Scene,
  window: Arc<winit::window::Window>,
  renderer: Box<dyn gfx::Renderer>,
  input_system: InputSystem,
  states: VecDeque<Box<dyn AppState>>,
  quit_requested: bool,
}

impl Application {
  fn init(&'static mut self) {
    // self.scene.observe::<crate::prefabs::Mesh, _>(|node, mesh| {
    //   node.add_component(gfx::Mesh::from_mesh(&mut self.render_device, mesh));
    //   println!("gfx::Mesh added!");
    // });
    // self.scene.observe::<crate::prefabs::GeomSphere, _>(|node, sphere| {
    //   node.add_component(gfx::Mesh::from_geomsphere(&self.render_device, sphere));
    //   println!("sphere added!");
    // });
  }
  fn transition(&mut self, trans: Transition) {
    match trans {
      Transition::None => (),
      Transition::Pop => {
        self.states.pop_back();
      }
      Transition::Push(state) => self.states.push_back(state),
      Transition::Switch(state) => {
        self.states.pop_back();
        self.states.push_back(state);
      }
      Transition::Quit => self.quit_requested = true,
    }
  }
  fn state(&mut self) -> &mut dyn AppState {
    self.states.back_mut().unwrap().as_mut()
  }
  fn app_data(&self) -> AppData {
    AppData {
      window: &self.window,
      scene: &self.scene,
    }
  }
  fn event(&mut self, event: &WindowEvent) {
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
      } => self.quit_requested = true,
      WindowEvent::Resized(physical_size) => self.on_resize(*physical_size),
      WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.on_resize(**new_inner_size),
      WindowEvent::KeyboardInput { .. }
      | WindowEvent::CursorMoved { .. }
      | WindowEvent::MouseInput { .. }
      | WindowEvent::MouseWheel { .. } => {
        app().input_system.handle_event(event);
        app().state().input(self.app_data(), &app().input_system);
      }
      _ => {}
    }
  }
  fn update(&mut self) {
    // Input update
    self.input_system.update();

    // Game update
    self.state().update(app().app_data());

    // Render
    self.renderer.render(app().app_data(), &self.render_device);

    self.input_system.reset_state();
  }
  pub fn run<A: AppState + 'static>(self, event_loop: winit::event_loop::EventLoop<()>) {
    unsafe {
      APP_INSTANCE = Some(self);
    }
    app().init();
    app().states.push_back(Box::new(A::init()));
    app().state().start(app().app_data());

    event_loop.run(move |event, _, control_flow| match event {
      // Event::RedrawRequested(window_id) if window_id == self.window.id() => {}
      Event::MainEventsCleared => {
        app().update();
        if app().quit_requested {
          unsafe {
            APP_INSTANCE = None;
          }
          *control_flow = ControlFlow::Exit;
        }
      }
      Event::WindowEvent {
        ref event,
        window_id,
      } if window_id == app().window.id() => {
        app().event(&event);
      }
      _ => {}
    });
  }
  fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.renderer.on_resize(&self.render_device, &new_size);
      self.state().resize(&new_size);
    }
  }
}
impl Drop for Application {
  fn drop(&mut self) {
    // Empty the states
    while !self.states.is_empty() {
      self.states.pop_back();
    }
  }
}
