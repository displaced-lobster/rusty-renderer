#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _position_padding: u32,
    pub color: [f32; 3],
    pub _color_padding: u32,
}
