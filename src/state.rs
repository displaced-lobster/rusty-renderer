use anyhow::Result;
use cgmath::{
  Deg,
  InnerSpace,
  Quaternion,
  Rotation3,
  Vector3,
  Zero,
};
use winit::{
  event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode},
  window::Window,
};
use wgpu::util::DeviceExt;

use crate::{
  camera::{CameraController, CameraRig, OrbitCamera, OrbitCameraController},
  instance::Instance,
  model::{Model, ModelPrimitive},
  render::Renderer,
};

const NUM_INSTANCES_PER_ROW: u32 = 1;

pub struct State {
  camera_rig: CameraRig<OrbitCamera, OrbitCameraController>,
  config: wgpu::SurfaceConfiguration,
  cube_model: Model,
  device: wgpu::Device,
  instance_buffer: wgpu::Buffer,
  mouse_pressed: bool,
  models: Vec<Model>,
  queue: wgpu::Queue,
  renderer: Renderer,
  pub size: winit::dpi::PhysicalSize<u32>,
  surface: wgpu::Surface,
}

impl State {
  pub async fn new(window: &Window) -> Self {
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance.request_adapter(
      &wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
      },
    ).await.unwrap();
    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        features: wgpu::Features::empty(),
        limits: wgpu::Limits::default(),
        label: None,
      },
      None,
    ).await.unwrap();
    let size = window.inner_size();
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface.get_preferred_format(&adapter).unwrap(),
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &config);

    let camera_rig = CameraRig::new((0.0, 5.0, 10.0));

    let mut renderer = Renderer::new(&device, &config);

    renderer.update_camera_uniform(&camera_rig.camera);

    let res_dir = std::path::Path::new(env!("OUT_DIR")).join("res");
    let cube_model = Model::load(
      &device,
      res_dir.join("cube.obj"),
    ).unwrap();

    let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
      (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        let position = Vector3 { x: x as f32, y: 0.0, z: z as f32 };
        let rotation = if position.is_zero() {
          Quaternion::from_axis_angle(
            Vector3::unit_z(),
            Deg(0.0),
          )
        } else {
          Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
        };

        Instance {
          position,
          rotation,
        }
      })
    }).collect::<Vec<_>>();
    let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage: wgpu::BufferUsages::VERTEX,
      }
    );

    Self {
      camera_rig,
      config,
      cube_model,
      device,
      instance_buffer,
      models: Vec::<Model>::new(),
      mouse_pressed: false,
      queue,
      renderer,
      size,
      surface,
    }
  }

  pub fn add_model_primitive(&mut self, primitive: ModelPrimitive, size: f32) {
    let model = match primitive {
      ModelPrimitive::Cube => Model::cube(&self.device, size),
      ModelPrimitive::Plane => Model::plane(&self.device, size),
    };

    self.models.push(model);
  }

  pub fn add_surface(&mut self, count: u32, size: f32, height_max: f32) {
    let model = Model::surface(&self.device, count, size, height_max);

    self.models.push(model);
  }

  pub fn input(&mut self, event: &DeviceEvent) -> bool {
    match event {
      DeviceEvent::Key(
        KeyboardInput {
          virtual_keycode: Some(key),
          state,
          ..
        }
      ) => {
        match (*key, *state) {
          (VirtualKeyCode::L, ElementState::Pressed) => {
            self.renderer.toggle_light_render();
          }
          (VirtualKeyCode::R, ElementState::Pressed) => {
            self.renderer.toggle_light_rotation();
          }
          _ => {
            self.camera_rig.controller.process_keyboard(*key, *state);
          }
        }
        true
      }
      DeviceEvent::MouseWheel { delta, .. } => {
        self.camera_rig.controller.process_scroll(delta);
        true
      }
      DeviceEvent::Button {
        button: 1,
        state,
      } => {
        self.mouse_pressed = *state == ElementState::Pressed;
        true
      }
      DeviceEvent::MouseMotion { delta } => {
        if self.mouse_pressed {
          self.camera_rig.controller.process_mouse(delta.0, delta.1);
        }
        true
      }
      _ => false
    }
  }

  pub fn prompt_for_file(&mut self) -> Result<()> {
    if let nfd::Response::Okay(path) = nfd::open_file_dialog(None, None)? {
      self.models.push(Model::load(&self.device, path)?);
    }
    Ok(())
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_frame()?.output;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    self.renderer.render(
      &self.device,
      &self.queue,
      &view,
      &self.cube_model,
      &self.models,
      &self.instance_buffer,
    );

    Ok(())
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
      self.renderer.resize(&self.device, &self.config);
    }
  }

  pub fn update(&mut self, dt: std::time::Duration) {
    self.camera_rig.controller.update_camera(&mut self.camera_rig.camera, dt);
    self.renderer.update_camera_uniform(&self.camera_rig.camera);
    self.renderer.update(&self.queue, dt);
  }
}
