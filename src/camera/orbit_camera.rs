use cgmath::{
  EuclideanSpace,
  InnerSpace,
  Matrix4,
  Point3,
  Vector3,
};
use std::time::Duration;
use winit::{
  dpi::PhysicalPosition,
  event::{ElementState, MouseScrollDelta, VirtualKeyCode},
};

use crate::camera::{Camera, CameraController};

#[derive(Debug)]
pub struct OrbitCamera {
  eye: Point3<f32>,
  target: Point3<f32>,
  up: Vector3<f32>,
}

impl OrbitCamera {
  pub fn new(eye: Point3<f32>) -> Self {
    let target = Point3::origin();
    let up = Vector3::unit_y();

    Self {
      eye,
      target,
      up,
    }
  }
}

impl Camera for OrbitCamera {
  fn from_position(position: Point3<f32>) -> Self {
    Self::new(position)
  }

  fn get_position(&self) -> Point3<f32> {
    return self.eye
  }

  fn projection(&self) -> Matrix4<f32> {
    Matrix4::look_to_rh(self.eye, self.target - self.eye, self.up)
  }
}

#[derive(Debug)]
pub struct OrbitCameraController {
  amount_left: f32,
  amount_right: f32,
  amount_forward: f32,
  amount_backward: f32,
  amount_up: f32,
  amount_down: f32,
  rotate_horizontal: f32,
  rotate_vertical: f32,
  scroll: f32,
  speed: f32,
  sensitivity: f32,
}

impl OrbitCameraController {
  pub fn new(speed: f32, sensitivity: f32) -> Self {
    Self {
      amount_left: 0.0,
      amount_right: 0.0,
      amount_forward: 0.0,
      amount_backward: 0.0,
      amount_up: 0.0,
      amount_down: 0.0,
      rotate_horizontal: 0.0,
      rotate_vertical: 0.0,
      scroll: 0.0,
      speed,
      sensitivity,
    }
  }
}

impl CameraController<OrbitCamera> for OrbitCameraController {
  fn default() -> Self {
    Self::new(4.0, 0.05)
  }

  fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
    let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };

    match key {
      VirtualKeyCode::W | VirtualKeyCode::Up => {
        self.amount_forward = amount;
        true
      }
      VirtualKeyCode::S | VirtualKeyCode::Down => {
        self.amount_backward = amount;
        true
      }
      VirtualKeyCode::A | VirtualKeyCode::Left => {
        self.amount_left = amount;
        true
      }
      VirtualKeyCode::D | VirtualKeyCode::Right => {
        self.amount_right = amount;
        true
      }
      VirtualKeyCode::Space => {
        self.amount_up = amount;
        true
      }
      VirtualKeyCode::LShift => {
        self.amount_down = amount;
        true
      }
      _ => false,
    }
  }

  fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
    self.rotate_horizontal = mouse_dx as f32;
    self.rotate_vertical = mouse_dy as f32;
  }

  fn process_scroll(&mut self, delta: &MouseScrollDelta) {
    self.scroll = match delta {
      MouseScrollDelta::LineDelta(_, scroll) => *scroll * -0.1,
      MouseScrollDelta::PixelDelta(PhysicalPosition{
        y: scroll,
        ..
      }) => *scroll as f32,
    };
  }

  fn update_camera(&mut self, camera: &mut OrbitCamera, dt: Duration) {
    let dt = dt.as_secs_f32();
    let forward = camera.target - camera.eye;
    let forward_norm = forward.normalize();
    let forward_mag = forward.magnitude();
    let right = forward_norm.cross(camera.up);
    let rotation_speed = self.sensitivity * forward_mag;
    let rotation_vector = (forward + rotation_speed * (right * self.rotate_horizontal - camera.up * self.rotate_vertical)).normalize();

    camera.eye = camera.target - rotation_vector * forward_mag;

    camera.eye += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
    camera.eye += right * (self.amount_right - self.amount_left) * self.speed * dt;

    camera.eye += self.scroll * forward_norm;


    self.rotate_horizontal = 0.0;
    self.rotate_vertical = 0.0;
    self.scroll = 0.0;
  }
}

