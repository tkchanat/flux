// extern crate alloc;
// use crate::gfx::{Backend, Buffer, BufferContents, BufferUsage, Format};
// use bytemuck::Pod;
// use std::sync::Arc;
// use vulkano::buffer::CpuAccessibleBuffer;

// pub struct Vulkan {}
// impl Backend for Vulkan {
//   type Device = VulkanDevice;
//   type Swapchain = VulkanSwapchain;
//   type Buffer = VulkanBuffer;
//   type Texture = VulkanTexture;
//   type Sampler = ();
//   type Descriptor = ();
//   type RenderPass = VulkanRenderPass;
//   type GraphicsPipeline = VulkanGraphicsPipeline;
//   type CommandList = VulkanCommandList<Self>;

//   fn create_device(
//     window: Option<Arc<winit::window::Window>>,
//   ) -> (Self::Device, Option<Self::Swapchain>) {
//     use vulkano::device::{
//       physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
//     };
//     use vulkano::image::{view::ImageView, ImageUsage};
//     use vulkano::instance::{Instance, InstanceCreateInfo};
//     use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo};
//     use vulkano::swapchain::{Swapchain, SwapchainCreateInfo};
//     use vulkano::VulkanLibrary;

//     // Creating an instance
//     let library = VulkanLibrary::new().expect("No local Vulkan library/DLL");
//     println!("Vulkan {:?}", library.api_version());

//     #[cfg(windows)]
//     let mut instance_create_info = InstanceCreateInfo::default();

//     #[cfg(target_os = "macos")]
//     let mut instance_create_info = InstanceCreateInfo {
//       enumerate_portability: true,
//       ..Default::default()
//     };

//     if window.is_some() {
//       let required_extensions = vulkano_win::required_extensions(&library);
//       instance_create_info.enabled_extensions = required_extensions;
//     }
//     let instance =
//       Instance::new(library, instance_create_info).expect("Failed to create vulkan instance");

//     let surface = window.and_then(|window| {
//       Some(
//         vulkano_win::create_surface_from_handle(window, instance.clone())
//           .expect("Unable to create surface from window"),
//       )
//     });

//     // Enumerating physical devices
//     let device_extensions = DeviceExtensions {
//       khr_swapchain: true,
//       ..DeviceExtensions::empty()
//     };
//     let (physical, queue_family_index) = instance
//       .enumerate_physical_devices()
//       .expect("Could not enumerate devices")
//       .filter(|p| p.supported_extensions().contains(&device_extensions))
//       .filter_map(|p| {
//         p.queue_family_properties()
//           .iter()
//           .enumerate()
//           // Find the first first queue family that is suitable.
//           // If none is found, `None` is returned to `filter_map`,
//           // which disqualifies this physical device.
//           .position(|(i, q)| {
//             q.queue_flags.graphics
//               && (surface.is_some()
//                 && p
//                   .surface_support(i as u32, surface.as_ref().unwrap())
//                   .unwrap_or(false))
//           })
//           .map(|q| (p, q as u32))
//       })
//       .min_by_key(|(p, _)| match p.properties().device_type {
//         PhysicalDeviceType::DiscreteGpu => 0,
//         PhysicalDeviceType::IntegratedGpu => 1,
//         PhysicalDeviceType::VirtualGpu => 2,
//         PhysicalDeviceType::Cpu => 3,

//         // Note that there exists `PhysicalDeviceType::Other`, however,
//         // `PhysicalDeviceType` is a non-exhaustive enum. Thus, one should
//         // match wildcard `_` to catch all unknown device types.
//         _ => 4,
//       })
//       .expect("No devices available");
//     println!(
//       "Found a queue family with {:?} queue(s)",
//       physical.queue_family_properties()[queue_family_index as usize].queue_count
//     );

//     let (device, mut queues) = Device::new(
//       physical.clone(),
//       DeviceCreateInfo {
//         // here we pass the desired queue family to use by index
//         queue_create_infos: vec![QueueCreateInfo {
//           queue_family_index,
//           ..Default::default()
//         }],
//         enabled_extensions: device_extensions,
//         ..Default::default()
//       },
//     )
//     .expect("Failed to create device");
//     let queue = queues.next().unwrap();

//     // Creating the swapchain
//     let swapchain = surface.and_then(|surface| {
//       let caps = physical
//         .surface_capabilities(&surface, Default::default())
//         .expect("Failed to get surface capabilities");
//       let dimensions = surface.window().inner_size();
//       let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
//       let image_format = Some(
//         physical
//           .surface_formats(&surface, Default::default())
//           .unwrap()[0]
//           .0,
//       );

//       let (swapchain, images) = Swapchain::new(
//         device.clone(),
//         surface.clone(),
//         SwapchainCreateInfo {
//           min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
//           image_format,
//           image_extent: dimensions.into(),
//           image_usage: ImageUsage {
//             color_attachment: true, // What the images are going to be used for
//             ..Default::default()
//           },
//           composite_alpha,
//           ..Default::default()
//         },
//       )
//       .unwrap();

//       let render_pass = vulkano::single_pass_renderpass!(
//           device.clone(),
//           attachments: {
//               color: {
//                   load: Clear,
//                   store: Store,
//                   format: swapchain.image_format(),  // set the format the same as the swapchain
//                   samples: 1,
//               }
//           },
//           pass: {
//               color: [color],
//               depth_stencil: {}
//           }
//       )
//       .unwrap();

//       let framebuffers = images
//         .iter()
//         .map(|image| {
//           let view = ImageView::new_default(image.clone()).unwrap();
//           Framebuffer::new(
//             render_pass.clone(),
//             FramebufferCreateInfo {
//               attachments: vec![view],
//               ..Default::default()
//             },
//           )
//           .unwrap()
//         })
//         .collect::<Vec<_>>();

//       Some(VulkanSwapchain {
//         handle: swapchain,
//         surface,
//         images,
//         render_pass,
//         framebuffers,
//       })
//     });

//     (
//       VulkanDevice {
//         instance,
//         // physical,
//         device,
//         queue,
//         queue_family_index,
//       },
//       swapchain,
//     )
//   }

//   fn create_buffer_with_init<T: BufferContents + Pod>(
//     device: &Self::Device,
//     usage: BufferUsage,
//     data: T,
//   ) -> Self::Buffer {
//     use vulkano::buffer::CpuAccessibleBuffer;

//     let buffer =
//       CpuAccessibleBuffer::<T>::from_data(device.device.clone(), usage.into(), false, data)
//         .expect("Failed to create buffer");
//     let access = buffer.clone();
//     VulkanBuffer {
//       handle: buffer,
//       access,
//     }
//   }

//   fn update_buffer<T: BufferContents + Pod, F: FnMut(&mut T)>(
//     device: &Self::Device,
//     buffer: &Self::Buffer,
//     mut f: F,
//   ) {
//     if let Some(buffer) = buffer
//       .handle
//       .as_any()
//       .downcast_ref::<CpuAccessibleBuffer<T>>()
//     {
//       let mut content = buffer.write().unwrap();
//       f(&mut content);
//     } else {
//       panic!();
//     }
//   }

//   fn copy_buffer_to_buffer(device: &Self::Device, src: &Self::Buffer, dst: &Self::Buffer) {
//     use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo};
//     use vulkano::sync::{self, GpuFuture};

//     let mut builder = AutoCommandBufferBuilder::primary(
//       device.device.clone(),
//       device.queue_family_index,
//       CommandBufferUsage::OneTimeSubmit,
//     )
//     .unwrap();
//     builder
//       .copy_buffer(CopyBufferInfo::buffers(
//         src.access.clone(),
//         dst.access.clone(),
//       ))
//       .unwrap();
//     let command_buffer = builder.build().unwrap();

//     let future = sync::now(device.device.clone())
//       .then_execute(device.queue.clone(), command_buffer)
//       .unwrap()
//       .then_signal_fence_and_flush() // same as signal fence, and then flush
//       .unwrap();
//     future.wait(Some(instant::Duration::new(5, 0))).unwrap(); // None is an optional timeout
//   }

//   fn create_texture(
//     device: &Self::Device,
//     extent: (u32, u32, u32),
//     format: Format,
//   ) -> Self::Texture {
//     use vulkano::image::{view::ImageView, ImageDimensions, ImageLayout, StorageImage};

//     let image = StorageImage::new(
//       device.device.clone(),
//       ImageDimensions::Dim2d {
//         width: extent.0,
//         height: extent.1,
//         array_layers: extent.2, // images can be arrays of layers
//       },
//       format.into(),
//       Some(device.queue_family_index),
//     )
//     .unwrap();
//     let access = image.clone();
//     let view = ImageView::new_default(image.clone()).unwrap();
//     let layout = ImageLayout::General;

//     VulkanTexture {
//       handle: image,
//       access,
//       view,
//       format: format.into(),
//       layout,
//     }
//   }

//   fn create_render_pass(
//     device: &Self::Device,
//     color_attachments: &[&Self::Texture],
//     depth_attachment: Option<&Self::Texture>,
//   ) -> Self::RenderPass {
//     use vulkano::image::{ImageLayout, SampleCount};
//     use vulkano::render_pass::{
//       AttachmentDescription, AttachmentReference, LoadOp, RenderPass, RenderPassCreateInfo,
//       StoreOp, SubpassDescription,
//     };

//     // let attachments = color_attachments.iter().chain(depth_attachment.iter());
//     let color_attachment_descriptions = color_attachments
//       .iter()
//       .map(|color| AttachmentDescription {
//         format: Some(color.format),
//         samples: SampleCount::Sample1,
//         load_op: LoadOp::Clear,
//         store_op: StoreOp::Store,
//         stencil_load_op: LoadOp::DontCare,
//         stencil_store_op: StoreOp::DontCare,
//         initial_layout: color.layout,
//         final_layout: ImageLayout::ColorAttachmentOptimal,
//         ..Default::default()
//       })
//       .collect::<Vec<_>>();
//     let depth_attachment_descriptions = depth_attachment.and_then(|depth| {
//       Some(AttachmentDescription {
//         format: Some(depth.format),
//         samples: SampleCount::Sample1,
//         load_op: LoadOp::Clear,
//         store_op: StoreOp::Store,
//         stencil_load_op: LoadOp::DontCare,
//         stencil_store_op: StoreOp::DontCare,
//         initial_layout: depth.layout,
//         final_layout: ImageLayout::DepthStencilAttachmentOptimal,
//         ..Default::default()
//       })
//     });
//     let attachments = color_attachment_descriptions
//       .into_iter()
//       .chain(depth_attachment_descriptions.into_iter())
//       .collect::<Vec<_>>();

//     let subpasses = Vec::from_iter([SubpassDescription {
//       view_mask: 0,
//       input_attachments: Vec::new(),
//       color_attachments: color_attachments
//         .iter()
//         .enumerate()
//         .map(|(i, color)| {
//           Some(AttachmentReference {
//             attachment: i as u32,
//             layout: ImageLayout::ColorAttachmentOptimal,
//             ..Default::default()
//           })
//         })
//         .collect::<Vec<_>>(),
//       resolve_attachments: Vec::new(),
//       depth_stencil_attachment: depth_attachment.and_then(|depth| {
//         Some(AttachmentReference {
//           attachment: (attachments.len() - 1) as u32,
//           layout: ImageLayout::DepthStencilAttachmentOptimal,
//           ..Default::default()
//         })
//       }),
//       preserve_attachments: Vec::new(),
//       ..Default::default()
//     }]);

//     let create_info = RenderPassCreateInfo {
//       attachments,
//       subpasses,
//       dependencies: Vec::new(),
//       correlated_view_masks: Vec::new(),
//       ..Default::default()
//     };
//     let render_pass =
//       RenderPass::new(device.device.clone(), create_info).expect("Unable to build render pass");
//     // let render_pass = vulkano::single_pass_renderpass!(device.device.clone(),
//     //     attachments: {
//     //         color: {
//     //             load: Clear,
//     //             store: Store,
//     //             format: Format::R8G8B8A8_UNORM.into(),
//     //             samples: 1,
//     //         }
//     //     },
//     //     pass: {
//     //         color: [color],
//     //         depth_stencil: {}
//     //     }
//     // )
//     // .unwrap();

//     VulkanRenderPass {
//       handle: render_pass,
//     }
//   }

//   fn create_graphics_pipeline(
//     device: &Self::Device,
//     render_pass: &Self::RenderPass,
//   ) -> Self::GraphicsPipeline {
//     use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
//     use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
//     use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
//     use vulkano::pipeline::GraphicsPipeline;
//     use vulkano::render_pass::Subpass;
//     use vulkano::shader::ShaderModule;

//     let (vs, vs_reflect) = unsafe {
//       let bytes = include_bytes!("../shaders/test.vert.spv");
//       (
//         ShaderModule::from_bytes(device.device.clone(), bytes).unwrap(),
//         spirv_reflect::ShaderModule::load_u8_data(bytes).unwrap(),
//       )
//     };
//     let (fs, fs_reflect) = unsafe {
//       let bytes = include_bytes!("../shaders/test.frag.spv");
//       (
//         ShaderModule::from_bytes(device.device.clone(), bytes).unwrap(),
//         spirv_reflect::ShaderModule::load_u8_data(bytes).unwrap(),
//       )
//     };

//     #[repr(C)]
//     #[derive(Default, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
//     struct Vertex {
//       position: [f32; 2],
//     }
//     vulkano::impl_vertex!(Vertex, position);

//     // More on this latter
//     let viewport = Viewport {
//       origin: [0.0, 0.0],
//       dimensions: [1024.0, 1024.0],
//       depth_range: 0.0..1.0,
//     };

//     let input_state = BuffersDefinition::new().vertex::<Vertex>();

//     let pipeline = GraphicsPipeline::start()
//       // Describes the layout of the vertex input and how should it behave
//       .vertex_input_state(input_state)
//       // A Vulkan shader can in theory contain multiple entry points, so we have to specify
//       // which one.
//       .vertex_shader(vs.entry_point("main").unwrap(), ())
//       // Indicate the type of the primitives (the default is a list of triangles)
//       .input_assembly_state(InputAssemblyState::new())
//       // Set the fixed viewport
//       .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
//       // Same as the vertex input, but this for the fragment input
//       .fragment_shader(fs.entry_point("main").unwrap(), ())
//       // This graphics pipeline object concerns the first pass of the render pass.
//       .render_pass(Subpass::from(render_pass.handle.clone(), 0).unwrap())
//       // Now that everything is specified, we call `build`.
//       .build(device.device.clone())
//       .unwrap();

//     VulkanGraphicsPipeline { handle: pipeline }
//   }

//   fn save_texture_to_disk(device: &Self::Device, texture: &Self::Texture) {
//     use vulkano::command_buffer::{
//       AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo,
//     };
//     use vulkano::sync::{self, GpuFuture};

//     let buf = CpuAccessibleBuffer::from_iter(
//       device.device.clone(),
//       vulkano::buffer::BufferUsage {
//         transfer_dst: true,
//         ..Default::default()
//       },
//       false,
//       (0..1024 * 1024 * 4).map(|_| 255u8),
//     )
//     .expect("Failed to create buffer");

//     let mut builder = AutoCommandBufferBuilder::primary(
//       device.device.clone(),
//       device.queue_family_index,
//       CommandBufferUsage::OneTimeSubmit,
//     )
//     .unwrap();
//     builder
//       .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
//         texture.access.clone(),
//         buf.clone(),
//       ))
//       .unwrap();

//     let command_buffer = builder.build().unwrap();

//     let future = sync::now(device.device.clone())
//       .then_execute(device.queue.clone(), command_buffer)
//       .unwrap()
//       .then_signal_fence_and_flush()
//       .unwrap();
//     future.wait(None).unwrap();

//     let buffer_content = buf.read().unwrap();
//     let image =
//       image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
//     image.save("image.png").unwrap();
//   }

//   /**********************/
//   /**** Command List ****/
//   /**********************/

//   fn create_command_list(device: &Self::Device) -> Self::CommandList {
//     use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};

//     let builder = AutoCommandBufferBuilder::primary(
//       device.device.clone(),
//       device.queue_family_index,
//       CommandBufferUsage::OneTimeSubmit,
//     )
//     .unwrap();

//     VulkanCommandList {
//       builder,
//       _pd: std::marker::PhantomData::default(),
//     }
//   }

//   fn begin_render_pass(
//     command_list: &mut Self::CommandList,
//     render_pass: &Self::RenderPass,
//     color_attachments: &[&Self::Texture],
//     depth_attachment: Option<&Self::Texture>,
//   ) {
//     use vulkano::command_buffer::{RenderPassBeginInfo, SubpassContents};
//     use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo};

//     let framebuffer = Framebuffer::new(
//       render_pass.handle.clone(),
//       FramebufferCreateInfo {
//         attachments: color_attachments
//           .iter()
//           .map(|color| color.view.clone())
//           .collect::<Vec<_>>(),
//         ..Default::default()
//       },
//     )
//     .unwrap();
//     command_list
//       .builder
//       .begin_render_pass(
//         RenderPassBeginInfo {
//           clear_values: color_attachments
//             .iter()
//             .map(|color| Some([0.0, 0.0, 0.0, 1.0].into()))
//             .collect::<Vec<_>>(),
//           ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
//         },
//         SubpassContents::Inline,
//       )
//       .unwrap();
//   }

//   fn end_render_pass(command_list: &mut Self::CommandList) {
//     command_list.builder.end_render_pass().unwrap();
//   }

//   fn bind_graphics_pipeline(
//     command_list: &mut Self::CommandList,
//     pipeline: &Self::GraphicsPipeline,
//   ) {
//     command_list
//       .builder
//       .bind_pipeline_graphics(pipeline.handle.clone());
//   }

//   fn bind_vertex_buffer(command_list: &mut Self::CommandList, buffer: &Self::Buffer) {
//     command_list
//       .builder
//       .bind_vertex_buffers(0, buffer.access.clone());
//   }

//   fn draw(
//     command_list: &mut Self::CommandList,
//     vertex_count: u32,
//     instance_count: u32,
//     first_vertex: u32,
//     first_instance: u32,
//   ) {
//     command_list
//       .builder
//       .draw(vertex_count, instance_count, first_vertex, first_instance)
//       .unwrap();
//   }

//   fn submit(device: &Self::Device, command_list: Self::CommandList) {
//     use vulkano::sync::{self, GpuFuture};

//     let command_buffer = command_list.builder.build().unwrap();
//     let future = sync::now(device.device.clone())
//       .then_execute(device.queue.clone(), command_buffer)
//       .unwrap()
//       .then_signal_fence_and_flush()
//       .unwrap();
//     future.wait(None).unwrap();
//   }
// }

// pub struct VulkanDevice {
//   instance: Arc<vulkano::instance::Instance>,
//   // physical: Arc<vulkano::device::physical::PhysicalDevice>,
//   device: Arc<vulkano::device::Device>,
//   queue: Arc<vulkano::device::Queue>,
//   queue_family_index: u32,
// }

// pub struct VulkanSwapchain {
//   handle: Arc<vulkano::swapchain::Swapchain<Arc<winit::window::Window>>>,
//   surface: Arc<vulkano::swapchain::Surface<Arc<winit::window::Window>>>,
//   images: Vec<Arc<vulkano::image::SwapchainImage<Arc<winit::window::Window>>>>,
//   render_pass: Arc<vulkano::render_pass::RenderPass>,
//   framebuffers: Vec<Arc<vulkano::render_pass::Framebuffer>>,
// }

// trait VulkanBufferTrait {
//   fn as_any(&self) -> &dyn std::any::Any;
// }
// impl<T: vulkano::buffer::BufferContents + ?Sized> VulkanBufferTrait
//   for vulkano::buffer::CpuAccessibleBuffer<T>
// {
//   fn as_any(&self) -> &dyn std::any::Any {
//     self as &dyn std::any::Any
//   }
// }
// pub struct VulkanBuffer {
//   handle: Arc<dyn VulkanBufferTrait>,
//   access: Arc<dyn vulkano::buffer::BufferAccess>,
// }

// pub struct VulkanTexture {
//   handle: Arc<vulkano::image::StorageImage>,
//   access: Arc<dyn vulkano::image::ImageAccess>,
//   view: Arc<dyn vulkano::image::ImageViewAbstract>,
//   format: vulkano::format::Format,
//   layout: vulkano::image::ImageLayout,
// }

// pub struct VulkanRenderPass {
//   handle: Arc<vulkano::render_pass::RenderPass>,
// }

// pub struct VulkanGraphicsPipeline {
//   handle: Arc<vulkano::pipeline::GraphicsPipeline>,
// }

// pub struct VulkanCommandList<B: Backend> {
//   builder: vulkano::command_buffer::AutoCommandBufferBuilder<
//     vulkano::command_buffer::PrimaryAutoCommandBuffer,
//   >,
//   _pd: std::marker::PhantomData<B>,
// }

// impl Into<vulkano::format::Format> for Format {
//   fn into(self) -> vulkano::format::Format {
//     match self {
//       Format::R4G4_UNORM_PACK8 => vulkano::format::Format::R4G4_UNORM_PACK8,
//       Format::R4G4B4A4_UNORM_PACK16 => vulkano::format::Format::R4G4B4A4_UNORM_PACK16,
//       Format::B4G4R4A4_UNORM_PACK16 => vulkano::format::Format::B4G4R4A4_UNORM_PACK16,
//       Format::R5G6B5_UNORM_PACK16 => vulkano::format::Format::R5G6B5_UNORM_PACK16,
//       Format::B5G6R5_UNORM_PACK16 => vulkano::format::Format::B5G6R5_UNORM_PACK16,
//       Format::R5G5B5A1_UNORM_PACK16 => vulkano::format::Format::R5G5B5A1_UNORM_PACK16,
//       Format::B5G5R5A1_UNORM_PACK16 => vulkano::format::Format::B5G5R5A1_UNORM_PACK16,
//       Format::A1R5G5B5_UNORM_PACK16 => vulkano::format::Format::A1R5G5B5_UNORM_PACK16,
//       Format::R8_UNORM => vulkano::format::Format::R8_UNORM,
//       Format::R8_SNORM => vulkano::format::Format::R8_SNORM,
//       Format::R8_USCALED => vulkano::format::Format::R8_USCALED,
//       Format::R8_SSCALED => vulkano::format::Format::R8_SSCALED,
//       Format::R8_UINT => vulkano::format::Format::R8_UINT,
//       Format::R8_SINT => vulkano::format::Format::R8_SINT,
//       Format::R8_SRGB => vulkano::format::Format::R8_SRGB,
//       Format::R8G8_UNORM => vulkano::format::Format::R8G8_UNORM,
//       Format::R8G8_SNORM => vulkano::format::Format::R8G8_SNORM,
//       Format::R8G8_USCALED => vulkano::format::Format::R8G8_USCALED,
//       Format::R8G8_SSCALED => vulkano::format::Format::R8G8_SSCALED,
//       Format::R8G8_UINT => vulkano::format::Format::R8G8_UINT,
//       Format::R8G8_SINT => vulkano::format::Format::R8G8_SINT,
//       Format::R8G8_SRGB => vulkano::format::Format::R8G8_SRGB,
//       Format::R8G8B8_UNORM => vulkano::format::Format::R8G8B8_UNORM,
//       Format::R8G8B8_SNORM => vulkano::format::Format::R8G8B8_SNORM,
//       Format::R8G8B8_USCALED => vulkano::format::Format::R8G8B8_USCALED,
//       Format::R8G8B8_SSCALED => vulkano::format::Format::R8G8B8_SSCALED,
//       Format::R8G8B8_UINT => vulkano::format::Format::R8G8B8_UINT,
//       Format::R8G8B8_SINT => vulkano::format::Format::R8G8B8_SINT,
//       Format::R8G8B8_SRGB => vulkano::format::Format::R8G8B8_SRGB,
//       Format::B8G8R8_UNORM => vulkano::format::Format::B8G8R8_UNORM,
//       Format::B8G8R8_SNORM => vulkano::format::Format::B8G8R8_SNORM,
//       Format::B8G8R8_USCALED => vulkano::format::Format::B8G8R8_USCALED,
//       Format::B8G8R8_SSCALED => vulkano::format::Format::B8G8R8_SSCALED,
//       Format::B8G8R8_UINT => vulkano::format::Format::B8G8R8_UINT,
//       Format::B8G8R8_SINT => vulkano::format::Format::B8G8R8_SINT,
//       Format::B8G8R8_SRGB => vulkano::format::Format::B8G8R8_SRGB,
//       Format::R8G8B8A8_UNORM => vulkano::format::Format::R8G8B8A8_UNORM,
//       Format::R8G8B8A8_SNORM => vulkano::format::Format::R8G8B8A8_SNORM,
//       Format::R8G8B8A8_USCALED => vulkano::format::Format::R8G8B8A8_USCALED,
//       Format::R8G8B8A8_SSCALED => vulkano::format::Format::R8G8B8A8_SSCALED,
//       Format::R8G8B8A8_UINT => vulkano::format::Format::R8G8B8A8_UINT,
//       Format::R8G8B8A8_SINT => vulkano::format::Format::R8G8B8A8_SINT,
//       Format::R8G8B8A8_SRGB => vulkano::format::Format::R8G8B8A8_SRGB,
//       Format::B8G8R8A8_UNORM => vulkano::format::Format::B8G8R8A8_UNORM,
//       Format::B8G8R8A8_SNORM => vulkano::format::Format::B8G8R8A8_SNORM,
//       Format::B8G8R8A8_USCALED => vulkano::format::Format::B8G8R8A8_USCALED,
//       Format::B8G8R8A8_SSCALED => vulkano::format::Format::B8G8R8A8_SSCALED,
//       Format::B8G8R8A8_UINT => vulkano::format::Format::B8G8R8A8_UINT,
//       Format::B8G8R8A8_SINT => vulkano::format::Format::B8G8R8A8_SINT,
//       Format::B8G8R8A8_SRGB => vulkano::format::Format::B8G8R8A8_SRGB,
//       Format::A8B8G8R8_UNORM_PACK32 => vulkano::format::Format::A8B8G8R8_UNORM_PACK32,
//       Format::A8B8G8R8_SNORM_PACK32 => vulkano::format::Format::A8B8G8R8_SNORM_PACK32,
//       Format::A8B8G8R8_USCALED_PACK32 => vulkano::format::Format::A8B8G8R8_USCALED_PACK32,
//       Format::A8B8G8R8_SSCALED_PACK32 => vulkano::format::Format::A8B8G8R8_SSCALED_PACK32,
//       Format::A8B8G8R8_UINT_PACK32 => vulkano::format::Format::A8B8G8R8_UINT_PACK32,
//       Format::A8B8G8R8_SINT_PACK32 => vulkano::format::Format::A8B8G8R8_SINT_PACK32,
//       Format::A8B8G8R8_SRGB_PACK32 => vulkano::format::Format::A8B8G8R8_SRGB_PACK32,
//       Format::A2R10G10B10_UNORM_PACK32 => vulkano::format::Format::A2R10G10B10_UNORM_PACK32,
//       Format::A2R10G10B10_SNORM_PACK32 => vulkano::format::Format::A2R10G10B10_SNORM_PACK32,
//       Format::A2R10G10B10_USCALED_PACK32 => vulkano::format::Format::A2R10G10B10_USCALED_PACK32,
//       Format::A2R10G10B10_SSCALED_PACK32 => vulkano::format::Format::A2R10G10B10_SSCALED_PACK32,
//       Format::A2R10G10B10_UINT_PACK32 => vulkano::format::Format::A2R10G10B10_UINT_PACK32,
//       Format::A2R10G10B10_SINT_PACK32 => vulkano::format::Format::A2R10G10B10_SINT_PACK32,
//       Format::A2B10G10R10_UNORM_PACK32 => vulkano::format::Format::A2B10G10R10_UNORM_PACK32,
//       Format::A2B10G10R10_SNORM_PACK32 => vulkano::format::Format::A2B10G10R10_SNORM_PACK32,
//       Format::A2B10G10R10_USCALED_PACK32 => vulkano::format::Format::A2B10G10R10_USCALED_PACK32,
//       Format::A2B10G10R10_SSCALED_PACK32 => vulkano::format::Format::A2B10G10R10_SSCALED_PACK32,
//       Format::A2B10G10R10_UINT_PACK32 => vulkano::format::Format::A2B10G10R10_UINT_PACK32,
//       Format::A2B10G10R10_SINT_PACK32 => vulkano::format::Format::A2B10G10R10_SINT_PACK32,
//       Format::R16_UNORM => vulkano::format::Format::R16_UNORM,
//       Format::R16_SNORM => vulkano::format::Format::R16_SNORM,
//       Format::R16_USCALED => vulkano::format::Format::R16_USCALED,
//       Format::R16_SSCALED => vulkano::format::Format::R16_SSCALED,
//       Format::R16_UINT => vulkano::format::Format::R16_UINT,
//       Format::R16_SINT => vulkano::format::Format::R16_SINT,
//       Format::R16_SFLOAT => vulkano::format::Format::R16_SFLOAT,
//       Format::R16G16_UNORM => vulkano::format::Format::R16G16_UNORM,
//       Format::R16G16_SNORM => vulkano::format::Format::R16G16_SNORM,
//       Format::R16G16_USCALED => vulkano::format::Format::R16G16_USCALED,
//       Format::R16G16_SSCALED => vulkano::format::Format::R16G16_SSCALED,
//       Format::R16G16_UINT => vulkano::format::Format::R16G16_UINT,
//       Format::R16G16_SINT => vulkano::format::Format::R16G16_SINT,
//       Format::R16G16_SFLOAT => vulkano::format::Format::R16G16_SFLOAT,
//       Format::R16G16B16_UNORM => vulkano::format::Format::R16G16B16_UNORM,
//       Format::R16G16B16_SNORM => vulkano::format::Format::R16G16B16_SNORM,
//       Format::R16G16B16_USCALED => vulkano::format::Format::R16G16B16_USCALED,
//       Format::R16G16B16_SSCALED => vulkano::format::Format::R16G16B16_SSCALED,
//       Format::R16G16B16_UINT => vulkano::format::Format::R16G16B16_UINT,
//       Format::R16G16B16_SINT => vulkano::format::Format::R16G16B16_SINT,
//       Format::R16G16B16_SFLOAT => vulkano::format::Format::R16G16B16_SFLOAT,
//       Format::R16G16B16A16_UNORM => vulkano::format::Format::R16G16B16A16_UNORM,
//       Format::R16G16B16A16_SNORM => vulkano::format::Format::R16G16B16A16_SNORM,
//       Format::R16G16B16A16_USCALED => vulkano::format::Format::R16G16B16A16_USCALED,
//       Format::R16G16B16A16_SSCALED => vulkano::format::Format::R16G16B16A16_SSCALED,
//       Format::R16G16B16A16_UINT => vulkano::format::Format::R16G16B16A16_UINT,
//       Format::R16G16B16A16_SINT => vulkano::format::Format::R16G16B16A16_SINT,
//       Format::R16G16B16A16_SFLOAT => vulkano::format::Format::R16G16B16A16_SFLOAT,
//       Format::R32_UINT => vulkano::format::Format::R32_UINT,
//       Format::R32_SINT => vulkano::format::Format::R32_SINT,
//       Format::R32_SFLOAT => vulkano::format::Format::R32_SFLOAT,
//       Format::R32G32_UINT => vulkano::format::Format::R32G32_UINT,
//       Format::R32G32_SINT => vulkano::format::Format::R32G32_SINT,
//       Format::R32G32_SFLOAT => vulkano::format::Format::R32G32_SFLOAT,
//       Format::R32G32B32_UINT => vulkano::format::Format::R32G32B32_UINT,
//       Format::R32G32B32_SINT => vulkano::format::Format::R32G32B32_SINT,
//       Format::R32G32B32_SFLOAT => vulkano::format::Format::R32G32B32_SFLOAT,
//       Format::R32G32B32A32_UINT => vulkano::format::Format::R32G32B32A32_UINT,
//       Format::R32G32B32A32_SINT => vulkano::format::Format::R32G32B32A32_SINT,
//       Format::R32G32B32A32_SFLOAT => vulkano::format::Format::R32G32B32A32_SFLOAT,
//       Format::R64_UINT => vulkano::format::Format::R64_UINT,
//       Format::R64_SINT => vulkano::format::Format::R64_SINT,
//       Format::R64_SFLOAT => vulkano::format::Format::R64_SFLOAT,
//       Format::R64G64_UINT => vulkano::format::Format::R64G64_UINT,
//       Format::R64G64_SINT => vulkano::format::Format::R64G64_SINT,
//       Format::R64G64_SFLOAT => vulkano::format::Format::R64G64_SFLOAT,
//       Format::R64G64B64_UINT => vulkano::format::Format::R64G64B64_UINT,
//       Format::R64G64B64_SINT => vulkano::format::Format::R64G64B64_SINT,
//       Format::R64G64B64_SFLOAT => vulkano::format::Format::R64G64B64_SFLOAT,
//       Format::R64G64B64A64_UINT => vulkano::format::Format::R64G64B64A64_UINT,
//       Format::R64G64B64A64_SINT => vulkano::format::Format::R64G64B64A64_SINT,
//       Format::R64G64B64A64_SFLOAT => vulkano::format::Format::R64G64B64A64_SFLOAT,
//       Format::B10G11R11_UFLOAT_PACK32 => vulkano::format::Format::B10G11R11_UFLOAT_PACK32,
//       Format::E5B9G9R9_UFLOAT_PACK32 => vulkano::format::Format::E5B9G9R9_UFLOAT_PACK32,
//       Format::D16_UNORM => vulkano::format::Format::D16_UNORM,
//       Format::X8_D24_UNORM_PACK32 => vulkano::format::Format::X8_D24_UNORM_PACK32,
//       Format::D32_SFLOAT => vulkano::format::Format::D32_SFLOAT,
//       Format::S8_UINT => vulkano::format::Format::S8_UINT,
//       Format::D16_UNORM_S8_UINT => vulkano::format::Format::D16_UNORM_S8_UINT,
//       Format::D24_UNORM_S8_UINT => vulkano::format::Format::D24_UNORM_S8_UINT,
//       Format::D32_SFLOAT_S8_UINT => vulkano::format::Format::D32_SFLOAT_S8_UINT,
//       Format::BC1_RGB_UNORM_BLOCK => vulkano::format::Format::BC1_RGB_UNORM_BLOCK,
//       Format::BC1_RGB_SRGB_BLOCK => vulkano::format::Format::BC1_RGB_SRGB_BLOCK,
//       Format::BC1_RGBA_UNORM_BLOCK => vulkano::format::Format::BC1_RGBA_UNORM_BLOCK,
//       Format::BC1_RGBA_SRGB_BLOCK => vulkano::format::Format::BC1_RGBA_SRGB_BLOCK,
//       Format::BC2_UNORM_BLOCK => vulkano::format::Format::BC2_UNORM_BLOCK,
//       Format::BC2_SRGB_BLOCK => vulkano::format::Format::BC2_SRGB_BLOCK,
//       Format::BC3_UNORM_BLOCK => vulkano::format::Format::BC3_UNORM_BLOCK,
//       Format::BC3_SRGB_BLOCK => vulkano::format::Format::BC3_SRGB_BLOCK,
//       Format::BC4_UNORM_BLOCK => vulkano::format::Format::BC4_UNORM_BLOCK,
//       Format::BC4_SNORM_BLOCK => vulkano::format::Format::BC4_SNORM_BLOCK,
//       Format::BC5_UNORM_BLOCK => vulkano::format::Format::BC5_UNORM_BLOCK,
//       Format::BC5_SNORM_BLOCK => vulkano::format::Format::BC5_SNORM_BLOCK,
//       Format::BC6H_UFLOAT_BLOCK => vulkano::format::Format::BC6H_UFLOAT_BLOCK,
//       Format::BC6H_SFLOAT_BLOCK => vulkano::format::Format::BC6H_SFLOAT_BLOCK,
//       Format::BC7_UNORM_BLOCK => vulkano::format::Format::BC7_UNORM_BLOCK,
//       Format::BC7_SRGB_BLOCK => vulkano::format::Format::BC7_SRGB_BLOCK,
//       Format::ETC2_R8G8B8_UNORM_BLOCK => vulkano::format::Format::ETC2_R8G8B8_UNORM_BLOCK,
//       Format::ETC2_R8G8B8_SRGB_BLOCK => vulkano::format::Format::ETC2_R8G8B8_SRGB_BLOCK,
//       Format::ETC2_R8G8B8A1_UNORM_BLOCK => vulkano::format::Format::ETC2_R8G8B8A1_UNORM_BLOCK,
//       Format::ETC2_R8G8B8A1_SRGB_BLOCK => vulkano::format::Format::ETC2_R8G8B8A1_SRGB_BLOCK,
//       Format::ETC2_R8G8B8A8_UNORM_BLOCK => vulkano::format::Format::ETC2_R8G8B8A8_UNORM_BLOCK,
//       Format::ETC2_R8G8B8A8_SRGB_BLOCK => vulkano::format::Format::ETC2_R8G8B8A8_SRGB_BLOCK,
//       Format::EAC_R11_UNORM_BLOCK => vulkano::format::Format::EAC_R11_UNORM_BLOCK,
//       Format::EAC_R11_SNORM_BLOCK => vulkano::format::Format::EAC_R11_SNORM_BLOCK,
//       Format::EAC_R11G11_UNORM_BLOCK => vulkano::format::Format::EAC_R11G11_UNORM_BLOCK,
//       Format::EAC_R11G11_SNORM_BLOCK => vulkano::format::Format::EAC_R11G11_SNORM_BLOCK,
//       Format::ASTC_4x4_UNORM_BLOCK => vulkano::format::Format::ASTC_4x4_UNORM_BLOCK,
//       Format::ASTC_4x4_SRGB_BLOCK => vulkano::format::Format::ASTC_4x4_SRGB_BLOCK,
//       Format::ASTC_5x4_UNORM_BLOCK => vulkano::format::Format::ASTC_5x4_UNORM_BLOCK,
//       Format::ASTC_5x4_SRGB_BLOCK => vulkano::format::Format::ASTC_5x4_SRGB_BLOCK,
//       Format::ASTC_5x5_UNORM_BLOCK => vulkano::format::Format::ASTC_5x5_UNORM_BLOCK,
//       Format::ASTC_5x5_SRGB_BLOCK => vulkano::format::Format::ASTC_5x5_SRGB_BLOCK,
//       Format::ASTC_6x5_UNORM_BLOCK => vulkano::format::Format::ASTC_6x5_UNORM_BLOCK,
//       Format::ASTC_6x5_SRGB_BLOCK => vulkano::format::Format::ASTC_6x5_SRGB_BLOCK,
//       Format::ASTC_6x6_UNORM_BLOCK => vulkano::format::Format::ASTC_6x6_UNORM_BLOCK,
//       Format::ASTC_6x6_SRGB_BLOCK => vulkano::format::Format::ASTC_6x6_SRGB_BLOCK,
//       Format::ASTC_8x5_UNORM_BLOCK => vulkano::format::Format::ASTC_8x5_UNORM_BLOCK,
//       Format::ASTC_8x5_SRGB_BLOCK => vulkano::format::Format::ASTC_8x5_SRGB_BLOCK,
//       Format::ASTC_8x6_UNORM_BLOCK => vulkano::format::Format::ASTC_8x6_UNORM_BLOCK,
//       Format::ASTC_8x6_SRGB_BLOCK => vulkano::format::Format::ASTC_8x6_SRGB_BLOCK,
//       Format::ASTC_8x8_UNORM_BLOCK => vulkano::format::Format::ASTC_8x8_UNORM_BLOCK,
//       Format::ASTC_8x8_SRGB_BLOCK => vulkano::format::Format::ASTC_8x8_SRGB_BLOCK,
//       Format::ASTC_10x5_UNORM_BLOCK => vulkano::format::Format::ASTC_10x5_UNORM_BLOCK,
//       Format::ASTC_10x5_SRGB_BLOCK => vulkano::format::Format::ASTC_10x5_SRGB_BLOCK,
//       Format::ASTC_10x6_UNORM_BLOCK => vulkano::format::Format::ASTC_10x6_UNORM_BLOCK,
//       Format::ASTC_10x6_SRGB_BLOCK => vulkano::format::Format::ASTC_10x6_SRGB_BLOCK,
//       Format::ASTC_10x8_UNORM_BLOCK => vulkano::format::Format::ASTC_10x8_UNORM_BLOCK,
//       Format::ASTC_10x8_SRGB_BLOCK => vulkano::format::Format::ASTC_10x8_SRGB_BLOCK,
//       Format::ASTC_10x10_UNORM_BLOCK => vulkano::format::Format::ASTC_10x10_UNORM_BLOCK,
//       Format::ASTC_10x10_SRGB_BLOCK => vulkano::format::Format::ASTC_10x10_SRGB_BLOCK,
//       Format::ASTC_12x10_UNORM_BLOCK => vulkano::format::Format::ASTC_12x10_UNORM_BLOCK,
//       Format::ASTC_12x10_SRGB_BLOCK => vulkano::format::Format::ASTC_12x10_SRGB_BLOCK,
//       Format::ASTC_12x12_UNORM_BLOCK => vulkano::format::Format::ASTC_12x12_UNORM_BLOCK,
//       Format::ASTC_12x12_SRGB_BLOCK => vulkano::format::Format::ASTC_12x12_SRGB_BLOCK,
//       Format::G8B8G8R8_422_UNORM => vulkano::format::Format::G8B8G8R8_422_UNORM,
//       Format::B8G8R8G8_422_UNORM => vulkano::format::Format::B8G8R8G8_422_UNORM,
//       Format::G8_B8_R8_3PLANE_420_UNORM => vulkano::format::Format::G8_B8_R8_3PLANE_420_UNORM,
//       Format::G8_B8R8_2PLANE_420_UNORM => vulkano::format::Format::G8_B8R8_2PLANE_420_UNORM,
//       Format::G8_B8_R8_3PLANE_422_UNORM => vulkano::format::Format::G8_B8_R8_3PLANE_422_UNORM,
//       Format::G8_B8R8_2PLANE_422_UNORM => vulkano::format::Format::G8_B8R8_2PLANE_422_UNORM,
//       Format::G8_B8_R8_3PLANE_444_UNORM => vulkano::format::Format::G8_B8_R8_3PLANE_444_UNORM,
//       Format::R10X6_UNORM_PACK16 => vulkano::format::Format::R10X6_UNORM_PACK16,
//       Format::R10X6G10X6_UNORM_2PACK16 => vulkano::format::Format::R10X6G10X6_UNORM_2PACK16,
//       Format::R10X6G10X6B10X6A10X6_UNORM_4PACK16 => {
//         vulkano::format::Format::R10X6G10X6B10X6A10X6_UNORM_4PACK16
//       }
//       Format::G10X6B10X6G10X6R10X6_422_UNORM_4PACK16 => {
//         vulkano::format::Format::G10X6B10X6G10X6R10X6_422_UNORM_4PACK16
//       }
//       Format::B10X6G10X6R10X6G10X6_422_UNORM_4PACK16 => {
//         vulkano::format::Format::B10X6G10X6R10X6G10X6_422_UNORM_4PACK16
//       }
//       Format::G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16 => {
//         vulkano::format::Format::G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16
//       }
//       Format::G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16 => {
//         vulkano::format::Format::G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16
//       }
//       Format::G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16 => {
//         vulkano::format::Format::G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16
//       }
//       Format::G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16 => {
//         vulkano::format::Format::G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16
//       }
//       Format::G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16 => {
//         vulkano::format::Format::G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16
//       }
//       Format::R12X4_UNORM_PACK16 => vulkano::format::Format::R12X4_UNORM_PACK16,
//       Format::R12X4G12X4_UNORM_2PACK16 => vulkano::format::Format::R12X4G12X4_UNORM_2PACK16,
//       Format::R12X4G12X4B12X4A12X4_UNORM_4PACK16 => {
//         vulkano::format::Format::R12X4G12X4B12X4A12X4_UNORM_4PACK16
//       }
//       Format::G12X4B12X4G12X4R12X4_422_UNORM_4PACK16 => {
//         vulkano::format::Format::G12X4B12X4G12X4R12X4_422_UNORM_4PACK16
//       }
//       Format::B12X4G12X4R12X4G12X4_422_UNORM_4PACK16 => {
//         vulkano::format::Format::B12X4G12X4R12X4G12X4_422_UNORM_4PACK16
//       }
//       Format::G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16 => {
//         vulkano::format::Format::G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16
//       }
//       Format::G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16 => {
//         vulkano::format::Format::G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16
//       }
//       Format::G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16 => {
//         vulkano::format::Format::G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16
//       }
//       Format::G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16 => {
//         vulkano::format::Format::G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16
//       }
//       Format::G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16 => {
//         vulkano::format::Format::G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16
//       }
//       Format::G16B16G16R16_422_UNORM => vulkano::format::Format::G16B16G16R16_422_UNORM,
//       Format::B16G16R16G16_422_UNORM => vulkano::format::Format::B16G16R16G16_422_UNORM,
//       Format::G16_B16_R16_3PLANE_420_UNORM => vulkano::format::Format::G16_B16_R16_3PLANE_420_UNORM,
//       Format::G16_B16R16_2PLANE_420_UNORM => vulkano::format::Format::G16_B16R16_2PLANE_420_UNORM,
//       Format::G16_B16_R16_3PLANE_422_UNORM => vulkano::format::Format::G16_B16_R16_3PLANE_422_UNORM,
//       Format::G16_B16R16_2PLANE_422_UNORM => vulkano::format::Format::G16_B16R16_2PLANE_422_UNORM,
//       Format::G16_B16_R16_3PLANE_444_UNORM => vulkano::format::Format::G16_B16_R16_3PLANE_444_UNORM,
//       Format::PVRTC1_2BPP_UNORM_BLOCK => vulkano::format::Format::PVRTC1_2BPP_UNORM_BLOCK,
//       Format::PVRTC1_4BPP_UNORM_BLOCK => vulkano::format::Format::PVRTC1_4BPP_UNORM_BLOCK,
//       Format::PVRTC2_2BPP_UNORM_BLOCK => vulkano::format::Format::PVRTC2_2BPP_UNORM_BLOCK,
//       Format::PVRTC2_4BPP_UNORM_BLOCK => vulkano::format::Format::PVRTC2_4BPP_UNORM_BLOCK,
//       Format::PVRTC1_2BPP_SRGB_BLOCK => vulkano::format::Format::PVRTC1_2BPP_SRGB_BLOCK,
//       Format::PVRTC1_4BPP_SRGB_BLOCK => vulkano::format::Format::PVRTC1_4BPP_SRGB_BLOCK,
//       Format::PVRTC2_2BPP_SRGB_BLOCK => vulkano::format::Format::PVRTC2_2BPP_SRGB_BLOCK,
//       Format::PVRTC2_4BPP_SRGB_BLOCK => vulkano::format::Format::PVRTC2_4BPP_SRGB_BLOCK,
//       Format::ASTC_4x4_SFLOAT_BLOCK => vulkano::format::Format::ASTC_4x4_SFLOAT_BLOCK,
//       Format::ASTC_5x4_SFLOAT_BLOCK => vulkano::format::Format::ASTC_5x4_SFLOAT_BLOCK,
//       Format::ASTC_5x5_SFLOAT_BLOCK => vulkano::format::Format::ASTC_5x5_SFLOAT_BLOCK,
//       Format::ASTC_6x5_SFLOAT_BLOCK => vulkano::format::Format::ASTC_6x5_SFLOAT_BLOCK,
//       Format::ASTC_6x6_SFLOAT_BLOCK => vulkano::format::Format::ASTC_6x6_SFLOAT_BLOCK,
//       Format::ASTC_8x5_SFLOAT_BLOCK => vulkano::format::Format::ASTC_8x5_SFLOAT_BLOCK,
//       Format::ASTC_8x6_SFLOAT_BLOCK => vulkano::format::Format::ASTC_8x6_SFLOAT_BLOCK,
//       Format::ASTC_8x8_SFLOAT_BLOCK => vulkano::format::Format::ASTC_8x8_SFLOAT_BLOCK,
//       Format::ASTC_10x5_SFLOAT_BLOCK => vulkano::format::Format::ASTC_10x5_SFLOAT_BLOCK,
//       Format::ASTC_10x6_SFLOAT_BLOCK => vulkano::format::Format::ASTC_10x6_SFLOAT_BLOCK,
//       Format::ASTC_10x8_SFLOAT_BLOCK => vulkano::format::Format::ASTC_10x8_SFLOAT_BLOCK,
//       Format::ASTC_10x10_SFLOAT_BLOCK => vulkano::format::Format::ASTC_10x10_SFLOAT_BLOCK,
//       Format::ASTC_12x10_SFLOAT_BLOCK => vulkano::format::Format::ASTC_12x10_SFLOAT_BLOCK,
//       Format::ASTC_12x12_SFLOAT_BLOCK => vulkano::format::Format::ASTC_12x12_SFLOAT_BLOCK,
//       Format::G8_B8R8_2PLANE_444_UNORM => vulkano::format::Format::G8_B8R8_2PLANE_444_UNORM,
//       Format::G10X6_B10X6R10X6_2PLANE_444_UNORM_3PACK16 => {
//         vulkano::format::Format::G10X6_B10X6R10X6_2PLANE_444_UNORM_3PACK16
//       }
//       Format::G12X4_B12X4R12X4_2PLANE_444_UNORM_3PACK16 => {
//         vulkano::format::Format::G12X4_B12X4R12X4_2PLANE_444_UNORM_3PACK16
//       }
//       Format::G16_B16R16_2PLANE_444_UNORM => vulkano::format::Format::G16_B16R16_2PLANE_444_UNORM,
//       Format::A4R4G4B4_UNORM_PACK16 => vulkano::format::Format::A4R4G4B4_UNORM_PACK16,
//       Format::A4B4G4R4_UNORM_PACK16 => vulkano::format::Format::A4B4G4R4_UNORM_PACK16,
//     }
//   }
// }

// impl Into<vulkano::buffer::BufferUsage> for BufferUsage {
//   fn into(self) -> vulkano::buffer::BufferUsage {
//     vulkano::buffer::BufferUsage {
//       transfer_dst: self.contains(BufferUsage::TRANSFER_DST),
//       transfer_src: self.contains(BufferUsage::TRANSFER_SRC),
//       uniform_buffer: self.contains(BufferUsage::UNIFORM_BUFFER),
//       uniform_texel_buffer: self.contains(BufferUsage::UNIFORM_TEXEL_BUFFER),
//       storage_buffer: self.contains(BufferUsage::STORAGE_BUFFER),
//       storage_texel_buffer: self.contains(BufferUsage::STORAGE_TEXEL_BUFFER),
//       index_buffer: self.contains(BufferUsage::INDEX_BUFFER),
//       vertex_buffer: self.contains(BufferUsage::VERTEX_BUFFER),
//       indirect_buffer: self.contains(BufferUsage::INDIRECT_BUFFER),
//       ..Default::default()
//     }
//   }
// }
