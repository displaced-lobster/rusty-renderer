use cgmath::{
    Deg,
    InnerSpace,
    Quaternion,
    Rotation3,
    Vector3,
    Zero,
};
use winit::{
    event::{DeviceEvent, ElementState, KeyboardInput},
    window::Window,
};
use wgpu::util::DeviceExt;

use crate::camera::{CameraController, CameraRig, CameraUniform, OrbitCamera, OrbitCameraController};
use crate::draw::{DrawLight, DrawModel};
use crate::instance::{Instance, InstanceRaw};
use crate::light::LightUniform;
use crate::model::{Model, ModelVertex, Vertex};
use crate::projection::Projection;
use crate::renderer::Renderer;
use crate::texture::Texture;

const NUM_INSTANCES_PER_ROW: u32 = 1;

pub struct State {
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_rig: CameraRig<OrbitCamera, OrbitCameraController>,
    camera_uniform: CameraUniform,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    light_bind_group: wgpu::BindGroup,
    light_buffer: wgpu::Buffer,
    light_uniform: LightUniform,
    mouse_pressed: bool,
    obj_model: Model,
    projection: Projection,
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
        let projection = Projection::new(config.width, config.height, Deg(45.0), 0.1, 100.0);
        let mut camera_uniform = CameraUniform::new();

        camera_uniform.update_view_proj(&camera_rig.camera, &projection);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),

                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_binding_group_layout"),
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
        };
        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: None,
        });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let mut renderer = Renderer::new(
            &device,
            &render_pipeline_layout,
            &config,
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc(), InstanceRaw::desc()],
        );
        let light_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
            push_constant_ranges: &[],
        });

        renderer.set_light_render_pipeline(
            &device,
            &light_pipeline_layout,
            config.format,
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc()],
        );

        let res_dir = std::path::Path::new(env!("OUT_DIR")).join("res");
        let obj_model = Model::load(
            &device,
            res_dir.join("pumpkin.obj"),
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
            camera_bind_group,
            camera_buffer,
            camera_rig,
            camera_uniform,
            config,
            device,
            instance_buffer,
            instances,
            light_bind_group,
            light_buffer,
            light_uniform,
            mouse_pressed: false,
            obj_model,
            projection,
            queue,
            renderer,
            size,
            surface,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.renderer.resize(&self.device, &self.config);
            self.projection.resize(new_size.width, new_size.height);
        }
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
                self.camera_rig.controller.process_keyboard(*key, *state);
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

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_rig.controller.update_camera(&mut self.camera_rig.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera_rig.camera, &self.projection);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        let old_position: Vector3<_> = self.light_uniform.position.into();

        self.light_uniform.position = (
            Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), Deg(60.0 * dt.as_secs_f32()))* old_position
        ).into();
        self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_frame()?.output;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: true,
                        },
                    }
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            if let Some(render_pipeline) = &self.renderer.light_render_pipeline {
                render_pass.set_pipeline(&render_pipeline);
                render_pass.draw_light_model(
                    &self.obj_model,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                );
            }

            render_pass.set_pipeline(&self.renderer.render_pipeline);
            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
