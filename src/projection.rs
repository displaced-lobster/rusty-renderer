use cgmath::{Matrix4, perspective, Rad};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);


pub struct Projection {
  aspect: f32,
  fovy: Rad<f32>,
  znear: f32,
  zfar: f32,
}

impl Projection {
  pub fn new<F: Into<Rad<f32>>>(
    width: u32,
    height: u32,
    fovy: F,
    znear: f32,
    zfar: f32,
  ) -> Self {
    Self {
      aspect: width as f32 / height as f32,
      fovy: fovy.into(),
      znear,
      zfar,
    }
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    self.aspect = width as f32 / height as f32;
  }

  pub fn calc_matrix(&self) -> Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
  }
}
