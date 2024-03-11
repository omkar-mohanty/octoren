use egui_wgpu::{
    self,
    wgpu::{self, util::DeviceExt, VertexBufferLayout},
    RenderState,
};
use std::sync::Arc;

use crate::{camera::CameraResources, texture::TextureResource};

pub trait Resource: Send + Sync + 'static {}

trait VertexTrait {
    fn desc() -> VertexBufferLayout<'static>;
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2], // NEW!
}


// lib.rs
#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    // Changed
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
];



#[rustfmt::skip]
const INDICES: &[u16] =&[ 
    0, 1, 4,
    1, 2, 4, 
    2, 3, 4
];

impl VertexTrait for Vertex {
     fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ]
        }
    }
}

pub fn create_render_pipeline(
    wgpu_render_state: &RenderState,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::RenderPipeline {
    let device = &wgpu_render_state.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custom3d"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./shader.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("custom3d"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("custom3d"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu_render_state.target_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub(crate) struct Renderer {
    render_state: RenderState,
}

impl Renderer {
    pub fn new(wgpu_render_state: &RenderState, pipeline: &Arc<wgpu::RenderPipeline>) -> Self {
        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(PipelineResources {
                pipeline: Arc::clone(&pipeline),
            });
        Self {
            render_state: wgpu_render_state.clone(),
        }
    }

    pub fn add_resource(&self, resource: impl Resource) {
        self.render_state
            .renderer
            .write()
            .callback_resources
            .insert(resource);
    }
}

pub struct CustomTriangleCallback {}

impl egui_wgpu::CallbackTrait for CustomTriangleCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        {
            let camera_render_resources: &mut CameraResources = resources.get_mut().unwrap();
            camera_render_resources.prepare(device, queue);
        }
        {
            let texture_render_resource: &mut TextureResource = resources.get_mut().unwrap();
            texture_render_resource.prepare(device, queue);
        }
        Vec::new()
    }

    fn paint<'a>(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let triangle_render_resources: &RenderResources = resources.get().unwrap();
        let pipeline_resources: &PipelineResources = resources.get().unwrap();
        let camera_render_resources: &CameraResources = resources.get().unwrap();
        let texture_render_resource: &TextureResource = resources.get().unwrap();

        pipeline_resources.paint(render_pass);
        camera_render_resources.paint(render_pass);
        texture_render_resource.paint(render_pass);
        triangle_render_resources.paint(render_pass);
        render_pass.draw_indexed(0..INDICES.len() as u32 as u32, 0, 0..1);
    }
}

struct PipelineResources {
    pub pipeline: Arc<wgpu::RenderPipeline>,
}

impl Resource for PipelineResources {}

impl PipelineResources {
    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(&self.pipeline);
    }
}
pub struct RenderResources {
    pub pipeline: Arc<wgpu::RenderPipeline>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Resource for RenderResources {}

impl RenderResources {
    pub fn new(wgpu_render_state: &RenderState, pipeline: &Arc<wgpu::RenderPipeline>) -> Self {
        let device = &wgpu_render_state.device;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline: Arc::clone(pipeline),
            vertex_buffer,
            index_buffer,
        }
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    }
}
