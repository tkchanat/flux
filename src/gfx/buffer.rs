use wgpu::util::DeviceExt;

use super::RenderDeviceOld;

#[derive(Clone, Debug)]
pub struct Buffer {
  pub(super) handle: usize,
}

pub struct VertexBuffer {
  pub(super) buffer: Buffer,
}

impl VertexBuffer {
  pub fn new(device: &mut RenderDeviceOld, data: &[u8]) -> Self {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: data,
      usage: wgpu::BufferUsages::VERTEX,
    });
    Self { buffer }
  }
}

pub struct IndexBuffer {
  pub(super) buffer: Buffer,
  pub(super) format: wgpu::IndexFormat,
}

impl IndexBuffer {
  pub fn new(device: &mut RenderDeviceOld, data: &[u8], format: wgpu::IndexFormat) -> Self {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: data,
      usage: wgpu::BufferUsages::INDEX,
    });
    Self { buffer, format }
  }
}

pub struct UniformBuffer<T> {
  pub(crate) buffer: Buffer,
  pub data: T,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> UniformBuffer<T> {
  pub fn new(device: &mut RenderDeviceOld) -> Self {
    let data = T::zeroed();
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: bytemuck::cast_slice(&[data]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    Self { buffer, data }
  }
}
