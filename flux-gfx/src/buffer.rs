use bytemuck::{
  bytes_of, cast_slice, try_cast_slice, try_cast_slice_mut, try_from_bytes, try_from_bytes_mut,
  Pod, PodCastError,
};
use std::ops::Deref;

bitflags::bitflags! {
  pub struct BufferUsage: u32 {
    const TRANSFER_SRC = 0b1;
    const TRANSFER_DST = 0b10;
    const UNIFORM_TEXEL_BUFFER = 0b100;
    const STORAGE_TEXEL_BUFFER = 0b1000;
    const UNIFORM_BUFFER = 0b1_0000;
    const STORAGE_BUFFER = 0b10_0000;
    const INDEX_BUFFER = 0b100_0000;
    const VERTEX_BUFFER = 0b1000_0000;
    const INDIRECT_BUFFER = 0b1_0000_0000;
  }
}

#[derive(Clone, Debug, Default)]
pub struct Buffer {
  pub(super) handle: Option<usize>,
}
impl Buffer {
  pub fn new(usage: BufferUsage, size: usize) -> Self {
    unsafe {
      if let Some(device) = crate::device::RENDER_DEVICE.as_ref() {
        device.create_buffer_raw(usage, size)
      } else {
        Self { handle: None }
      }
    }
  }

  pub fn map<T: bytemuck::Pod, F: Fn(&mut T)>(&self, f: F) {
    unsafe {
      if let Some(device) = crate::device::RENDER_DEVICE.as_ref() {
        device.map_buffer(&self, f);
      }
    }
  }
}

pub struct VertexBuffer {
  pub(super) buffer: Buffer,
}
impl VertexBuffer {
  pub fn new<B: bytemuck::Pod>(vertices: B) -> Self {
    Self::from_slice(&[vertices])
  }

  pub fn from_slice<B: bytemuck::Pod>(vertices: &[B]) -> Self {
    let buffer = unsafe {
      crate::device::RENDER_DEVICE.as_ref().map_or_else(
        || Buffer::default(),
        |device| device.create_buffer(BufferUsage::VERTEX_BUFFER, bytemuck::cast_slice(vertices)),
      )
    };
    Self { buffer }
  }
}
impl Deref for VertexBuffer {
  type Target = Buffer;
  fn deref(&self) -> &Self::Target {
    &self.buffer
  }
}

pub enum IndexFormat {
  U16,
  U32,
}

pub trait IndexType: bytemuck::Pod {
  fn format() -> IndexFormat;
}
impl IndexType for u16 {
  fn format() -> IndexFormat {
    IndexFormat::U16
  }
}
impl IndexType for u32 {
  fn format() -> IndexFormat {
    IndexFormat::U32
  }
}

pub struct IndexBuffer {
  pub(super) buffer: Buffer,
  pub(super) format: IndexFormat,
}
impl IndexBuffer {
  pub fn new<I: IndexType>(indices: &[I]) -> Self {
    let buffer = unsafe {
      crate::device::RENDER_DEVICE.as_ref().map_or_else(
        || Buffer::default(),
        |device| device.create_buffer(BufferUsage::INDEX_BUFFER, bytemuck::cast_slice(indices)),
      )
    };
    Self {
      buffer,
      format: I::format(),
    }
  }
}
impl Deref for IndexBuffer {
  type Target = Buffer;
  fn deref(&self) -> &Self::Target {
    &self.buffer
  }
}

pub struct UniformBuffer<T> {
  pub(crate) buffer: Buffer,
  pub data: T,
}
impl<T: bytemuck::Pod + bytemuck::Zeroable> UniformBuffer<T> {
  pub fn new() -> Self {
    let data = T::zeroed();
    let buffer = unsafe {
      crate::device::RENDER_DEVICE.as_ref().map_or_else(
        || Buffer::default(),
        |device| device.create_buffer(BufferUsage::UNIFORM_BUFFER, bytemuck::cast_slice(&[data])),
      )
    };
    Self { buffer, data }
  }
}
impl<T> Deref for UniformBuffer<T> {
  type Target = Buffer;
  fn deref(&self) -> &Self::Target {
    &self.buffer
  }
}