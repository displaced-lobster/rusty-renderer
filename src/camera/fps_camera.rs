use cgmath::{
  Deg,
  InnerSpace,
  Matrix4,
  Point3,
  Rad,
  Vector3,
};
use std::{
  f32::consts::FRAC_PI_2,
  time::Duration,
};
use winit::{
  dpi::PhysicalPosition,
  event::{ElementState, MouseScrollDelta, VirtualKeyCode},
};

use crate::camera::{Camera, CameraController};

#[derive(Debug)]
pub struct FPSCamera {
  pub position: Point3<f32>,
  pub yaw: Rad<f32>,
  pub pitch: Rad<f32>,
}

impl FPSCamera {
  pub fn new<
    V: Into<Point3<f32>>,
    Y: Into<Rad<f32>>,
    P: Into<Rad<f32>>,
  >(
    position: V,
    yaw: Y,
    pitch: P,
  ) -> Self {
    Self {
      position: position.into(),
      yaw: yaw.into(),
      pitch: pitch.into(),
    }
  }
}

impl Camera for FPSCamera {
  fn from_position(position: Point3<f32>) -> Self {
    Self::new(position, Deg(-90.0), Deg(-20.0))
  }

  fn get_position(&self) -> Point3<f32> {
    return self.position
  }

  fn projection(&self) -> Matrix4<f32> {
    Matrix4::look_to_rh(
      self.position,
      Vector3::new(
        self.yaw.0.cos(),
        self.pitch.0.sin(),
        self.yaw.0.sin(),
      ).normalize(),
      Vector3::unit_y(),
    )
  }
}

#[derive(Debug)]
pub struct FPSCameraController {
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

impl FPSCameraController {
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

impl CameraController<FPSCamera> for FPSCameraController {
  fn default() -> Self {
    Self::new(4.0, 4.0)
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

  fn update_camera(&mut self, camera: &mut FPSCamera, dt: Duration) {
    let dt = dt.as_secs_f32();

    let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
    let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
    let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

    camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
    camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

    let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
    let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

    camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
    self.scroll = 0.0;

    camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

    camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
    camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;
    self.rotate_horizontal = 0.0;
    self.rotate_vertical = 0.0;

    if camera.pitch < -Rad(FRAC_PI_2) {
      camera.pitch = -Rad(FRAC_PI_2);
    } else if camera.pitch > Rad(FRAC_PI_2) {
      camera.pitch = Rad(FRAC_PI_2);
    }
  }
}

