use std::sync::Arc;

use cgmath::Matrix4;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::{FramebufferAbstract, Framebuffer, RenderPass, RenderPassDesc, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain::Swapchain;
use vulkano::image::swapchain::SwapchainImage;
use winit::Window;
use vulkano::format::D32Sfloat;
use vulkano::image::attachment::AttachmentImage;
use vulkano::command_buffer::{AutoCommandBufferBuilder, AutoCommandBuffer, DynamicState};
use vulkano::pipeline::viewport::Viewport;
use vulkano::sampler::{Sampler, Filter, SamplerAddressMode, MipmapMode};

use geometry::VertexPositionNormalUVColor;
use renderpass::RenderPassClearedColorWithDepth;
use renderer::ChunkRenderQueueEntry;
use util::Transform;

use super::RenderPipeline;
use shader::default as DefaultShaders;
use registry::TextureRegistry;


pub struct ChunkRenderPipeline {
    device: Arc<Device>,
    vulkan_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    pub framebuffers: Option<Vec<Arc<FramebufferAbstract + Send + Sync>>>,
    renderpass: Arc<RenderPass<RenderPassClearedColorWithDepth>>,
    uniform_buffer_pool: CpuBufferPool<DefaultShaders::vertex::ty::Data>,
    sampler: Arc<Sampler>,
}


impl ChunkRenderPipeline {
    pub fn new(swapchain: &Swapchain<Window>, device: Arc<Device>) -> ChunkRenderPipeline {
        let vs = DefaultShaders::vertex::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = DefaultShaders::fragment::Shader::load(device.clone()).expect("failed to create shader module");

        let renderpass = Arc::new(
            RenderPassClearedColorWithDepth { color_format: swapchain.format() }
                .build_render_pass(device.clone())
                .unwrap()
        );

        let pipeline = Arc::new(GraphicsPipeline::start()
            .cull_mode_back()
            .vertex_input_single_buffer::<VertexPositionNormalUVColor>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .blend_alpha_blending()
            .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap());

        ChunkRenderPipeline {
            device: device.clone(),
            vulkan_pipeline: pipeline,
            framebuffers: None,
            renderpass,
            uniform_buffer_pool: CpuBufferPool::<DefaultShaders::vertex::ty::Data>::new(device.clone(), BufferUsage::all()),
            sampler: Sampler::new(device.clone(), Filter::Nearest, Filter::Nearest, MipmapMode::Nearest,
                                  SamplerAddressMode::Repeat, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
                                  0.0, 4.0, 0.0, 0.0).unwrap(),
        }
    }


    pub fn build_command_buffer(&self, image_num: usize, queue: &Arc<Queue>, dimensions: [u32; 2], transform: &Transform, view_mat: Matrix4<f32>, proj_mat: Matrix4<f32>, tex_registry: &TextureRegistry, render_queue: &Vec<ChunkRenderQueueEntry>) -> AutoCommandBuffer {
        let mut descriptor_sets = Vec::new();
        for entry in render_queue.iter() {
            let uniform_data = DefaultShaders::vertex::ty::Data {
                world: entry.transform.clone().into(),
                view: view_mat.into(),
                proj: proj_mat.into(),
                view_pos: transform.position.into(),
            };

            let subbuffer = self.uniform_buffer_pool.next(uniform_data).unwrap();
            descriptor_sets.push(Arc::new(::vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(self.vulkan_pipeline.clone(), 0)
                .add_sampled_image(tex_registry.get(&entry.material.albedo_map_name).unwrap().clone(), self.sampler.clone()).unwrap()
                .add_buffer(subbuffer).unwrap()
                .build().unwrap()
            ));
        };

        let mut cb = AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), queue.family())
            .unwrap()
            .begin_render_pass(
                self.framebuffers.as_ref().unwrap()[image_num].clone(), false,
                vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()]).unwrap();
        for (i, entry) in render_queue.iter().enumerate() {
            cb = cb.draw_indexed(self.vulkan_pipeline.clone(), DynamicState {
                line_width: None,
                viewports: Some(vec![Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }]),
                scissors: None,
            },
                                 vec![entry.vertex_group.vertex_buffer.as_ref().unwrap().clone()],
                                 entry.vertex_group.index_buffer.as_ref().unwrap().clone(),
                                 descriptor_sets[i].clone(), ()).unwrap();
        }
        let cb = cb.end_render_pass().unwrap()
            .build().unwrap();
        cb
    }
}


impl RenderPipeline for ChunkRenderPipeline {
    fn pipeline(&self) -> &Arc<GraphicsPipelineAbstract + Send + Sync> { &self.vulkan_pipeline }


    fn remove_framebuffers(&mut self) {
        self.framebuffers = None;
    }


    fn recreate_framebuffers(&mut self, images: &Vec<Arc<SwapchainImage<Window>>>, depth_buffer: &Arc<AttachmentImage<D32Sfloat>>) {
        let new_framebuffers = Some(images.iter().map(|image| {
            let arc: Arc<FramebufferAbstract + Send + Sync> = Arc::new(Framebuffer::start(self.renderpass.clone())
                .add(image.clone()).unwrap()
                .add(depth_buffer.clone()).unwrap()
                .build().unwrap());
            arc
        }).collect::<Vec<_>>());
        ::std::mem::replace(&mut self.framebuffers, new_framebuffers);
    }
}