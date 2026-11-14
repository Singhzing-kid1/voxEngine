use rapier3d::glamx::{u8vec4, uvec4};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    format::Format,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{
        GraphicsPipeline, Pipeline, PipelineBindPoint,
        graphics::viewport::{Scissor, Viewport},
    },
};

use super::{
    Element,
    shader::{fragment, vertex},
};

use crate::{
    common::{UiVertex, Updateable},
    engine::{Engine, Frame, Record, helpers},
    texture::Atlas,
};

use std::sync::Arc;

pub struct Ui {
    elements: Vec<Box<dyn Element>>,
    textures: Atlas,

    pipeline: Arc<GraphicsPipeline>,

    vertex_buffer: Option<Subbuffer<[UiVertex]>>,
    index_buffer: Option<Subbuffer<[u32]>>,
}

impl Ui {
    pub fn new(engine: &mut Engine) -> Self {
        let elements = Vec::new();
        let textures = Atlas::new(Format::R8G8B8A8_UNORM, 0f32, engine);

        let vertex_shader = vertex::load(engine.get_device()).unwrap();
        let fragment_shader = fragment::load(engine.get_device()).unwrap();

        let pipeline = helpers::create_graphics_pipeline(
            vertex_shader,
            fragment_shader,
            "main",
            textures.get_format(),
            engine.get_device(),
        );

        Self {
            elements,
            textures,
            pipeline,

            vertex_buffer: None,
            index_buffer: None,
        }
    }

    pub fn add_element(&mut self, engine: &mut Engine, element: Box<dyn Element>) {
        self.textures.add_image(
            element.get_texture().clone(),
            engine,
            true,
            Some(helpers::get_layout(self.pipeline.layout(), 0)),
        );

        self.elements.push(element);
    }
}

impl Record for Ui {
    fn init(&mut self, engine: &mut Engine) {
        let mut vertices: Vec<UiVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for e in self.elements.iter_mut() {
            let base = vertices.len() as u32;
            e.set_first_idx(indices.len() as u32);

            let (verts, inds, _) = e.get_tris();

            vertices.extend([
                UiVertex {
                    pos: verts[0].to_array(),
                    color: u8vec4(255, 255, 255, 255).to_array(),
                },
                UiVertex {
                    pos: verts[1].to_array(),
                    color: u8vec4(255, 255, 255, 255).to_array(),
                },
                UiVertex {
                    pos: verts[2].to_array(),
                    color: u8vec4(255, 255, 255, 255).to_array(),
                },
                UiVertex {
                    pos: verts[3].to_array(),
                    color: u8vec4(255, 255, 255, 255).to_array(),
                },
            ]);
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        let vertex_buffer = Buffer::from_iter(
            engine.get_memory_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();

        let index_buffer = Buffer::from_iter(
            engine.get_memory_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices,
        )
        .unwrap();

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
    }

    fn record(&mut self, frame: &mut Frame, engine: &mut Engine) {
        let screen_size = [
            engine.get_dimensions().0 as f32,
            engine.get_dimensions().1 as f32,
        ];

        let push_data = vertex::PC { screen_size };

        let builder = frame.get_builder_mut();

        builder
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: screen_size,
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
            .unwrap()
            .set_scissor(
                0,
                [Scissor {
                    offset: [0, 0],
                    extent: [
                        engine.get_dimensions().0 as u32,
                        engine.get_dimensions().1 as u32,
                    ],
                }]
                .into_iter()
                .collect(),
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .push_constants(self.pipeline.layout().clone(), 0, push_data)
            .unwrap()
            .bind_vertex_buffers(0, self.vertex_buffer.as_ref().unwrap().clone())
            .unwrap()
            .bind_index_buffer(self.index_buffer.as_ref().unwrap().clone())
            .unwrap();

        for e in self.elements.iter() {
            builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.pipeline.layout().clone(),
                    0,
                    self.textures
                        .get_texture(e.get_texture().to_str().unwrap().to_string())
                        .1
                        .unwrap()
                        .clone(),
                )
                .unwrap();

            let (_, _, first_index) = e.get_tris();

            unsafe {
                builder.draw_indexed(6, 1, first_index, 0, 0).unwrap();
            }
        }
    }
}
