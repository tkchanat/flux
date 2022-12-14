pub struct InputSystem {
  mouse_position: Option<(f64, f64)>,
  last_mouse_position: Option<(f64, f64)>,
  mouse_delta: (f64, f64),
  mouse_pressed: [bool; 3],
  scroll_delta: Option<(f32, f32)>,
  key_pressed: [bool; 163],
}

impl InputSystem {
  pub fn new() -> Self {
    Self {
      mouse_position: None,
      last_mouse_position: None,
      mouse_delta: (0.0, 0.0),
      mouse_pressed: [false; 3],
      scroll_delta: None,
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
  pub fn reset_state(&mut self) {
    self.scroll_delta = None;
  }
  pub fn handle_event(&mut self, event: &winit::event::WindowEvent) {
    match event {
      winit::event::WindowEvent::CursorMoved { position, .. } => {
        self.mouse_position = Some((position.x, position.y));
      }
      winit::event::WindowEvent::CursorLeft { .. } => {
        self.mouse_position = None;
      }
      winit::event::WindowEvent::MouseInput { state, button, .. } => {
        let pressed = *state == winit::event::ElementState::Pressed;
        match button {
          winit::event::MouseButton::Left => {
            self.mouse_pressed[0] = pressed;
          }
          winit::event::MouseButton::Right => {
            self.mouse_pressed[1] = pressed;
          }
          winit::event::MouseButton::Middle => {
            self.mouse_pressed[2] = pressed;
          }
          _ => (),
        }
      }
      winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
        winit::event::MouseScrollDelta::LineDelta(dx, dy) => self.scroll_delta = Some((*dx, *dy)),
        winit::event::MouseScrollDelta::PixelDelta(delta) => {
          let delta = (delta.x as f32 * 0.1, delta.y as f32 * 0.1);
          self.scroll_delta = Some(delta)
        }
      },
      winit::event::WindowEvent::KeyboardInput { input, .. } => {
        let pressed = input.state == winit::event::ElementState::Pressed;
        if let Some(keycode) = input.virtual_keycode {
          self.key_pressed[keycode as usize] = pressed;
        }
      }
      _ => {}
    }
  }
  pub fn is_mouse_pressed(&self, button: winit::event::MouseButton) -> bool {
    let index = match button {
      winit::event::MouseButton::Left => 0,
      winit::event::MouseButton::Right => 1,
      winit::event::MouseButton::Middle => 2,
      winit::event::MouseButton::Other(_) => panic!("Unsupported mouse button"),
    };
    self.mouse_pressed[index]
  }
  pub fn is_mouse_released(&self, button: winit::event::MouseButton) -> bool {
    !self.is_mouse_pressed(button)
  }
  pub fn is_key_pressed(&self, keycode: winit::event::VirtualKeyCode) -> bool {
    self.key_pressed[keycode as usize]
  }
  pub fn is_key_released(&self, keycode: winit::event::VirtualKeyCode) -> bool {
    !self.is_key_pressed(keycode)
  }
  pub fn mouse_delta(&self) -> (f64, f64) {
    self.mouse_delta
  }
  pub fn scroll_delta(&self) -> (f32, f32) {
    match self.scroll_delta {
      Some(delta) => delta,
      None => (0.0, 0.0),
    }
  }
}
