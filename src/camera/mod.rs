use cgmath::{
  Matrix4,
  Point3,
  SquareMatrix,
};
use std::time::Duration;
use winit::event::{
  ElementState,
  MouseScrollDelta,
  VirtualKeyCode,
};

use crate::projection::Projection;

pub mod fps_camera;
pub mod orbit_camera;

pub use fps_camera::{FPSCamera, FPSCameraController};
pub use orbit_camera::{OrbitCamera, OrbitCameraController};

pub trait Camera {
  fn from_position(position: Point3<f32>) -> Self;
  fn get_position(&self) -> Point3<f32>;
  fn projection(&self) -> Matrix4<f32>;
}

pub trait CameraController<C>
where
  C: Camera,
{
  fn default() -> Self;
  fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool;
  fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64);
  fn process_scroll(&mut self, delta: &MouseScrollDelta);
  fn update_camera(&mut self, camera: &mut C, dt: Duration);
}

pub struct CameraRig<C, CC>
where
  C: Camera,
  CC: CameraController<C>,
{
  pub camera: C,
  pub controller: CC,
}

impl<C, CC> CameraRig<C, CC>
where
  C: Camera,
  CC: CameraController<C>
{
  pub fn new<P: Into<Point3<f32>>>(position: P) -> Self {
    Self {
      camera: C::from_position(position.into()),
      controller: CC::default(),
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  view_position: [f32; 4],
  view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
  pub fn new() -> Self {
    Self {
      view_position: [0.0; 4],
      view_proj: Matrix4::identity().into(),
    }
  }

  pub fn update_view_proj<C: Camera>(&mut self, camera: &C, projection: &Projection) {
    self.view_position = camera.get_position().to_homogeneous().into();
    self.view_proj = (projection.calc_matrix() * camera.projection()).into();
  }
}
