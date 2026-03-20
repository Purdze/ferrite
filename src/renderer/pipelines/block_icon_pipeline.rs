use std::sync::{Arc, Mutex};

use ash::vk;
use glam::{Mat4, Vec3};
use gpu_allocator::vulkan::{Allocation, Allocator};

use crate::renderer::chunk::atlas::TextureAtlas;
use crate::renderer::chunk::mesher::ChunkVertex;
use crate::renderer::shader;
use crate::renderer::util;
use crate::world::block::model::{BakedQuad, Direction};

const MAX_ICON_VERTICES: usize = 12000;
const VERTEX_SIZE: usize = std::mem::size_of::<ChunkVertex>();

pub struct BlockIconRequest<'a> {
    pub screen_x: f32,
    pub screen_y: f32,
    pub size: f32,
    pub quads: &'a [BakedQuad],
}

pub struct BlockIconPipeline {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    atlas_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    atlas_set: vk::DescriptorSet,
    vertex_buffer: vk::Buffer,
    vertex_allocation: Allocation,
}

impl BlockIconPipeline {
    pub fn new(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        allocator: &Arc<Mutex<Allocator>>,
        atlas: &TextureAtlas,
    ) -> Self {
        let atlas_layout = util::create_descriptor_set_layout(
            device,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            vk::ShaderStageFlags::FRAGMENT,
        );

        let push_range = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: 64,
        }];
        let layouts = [atlas_layout];
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&layouts)
            .push_constant_ranges(&push_range);
        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None) }
            .expect("failed to create block icon pipeline layout");

        let pipeline = create_pipeline(device, render_pass, pipeline_layout);

        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
        }];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(1)
            .pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None) }
            .expect("failed to create block icon descriptor pool");

        let alloc_layouts = [atlas_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&alloc_layouts);
        let atlas_set = unsafe { device.allocate_descriptor_sets(&alloc_info) }
            .expect("failed to allocate block icon descriptor set")[0];

        let image_info = [vk::DescriptorImageInfo {
            sampler: atlas.sampler,
            image_view: atlas.view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        }];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(atlas_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info);
        unsafe { device.update_descriptor_sets(&[write], &[]) };

        let (vertex_buffer, vertex_allocation) = util::create_host_buffer(
            device,
            allocator,
            (MAX_ICON_VERTICES * VERTEX_SIZE) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            "block_icon_vertices",
        );

        Self {
            pipeline,
            pipeline_layout,
            atlas_layout,
            descriptor_pool,
            atlas_set,
            vertex_buffer,
            vertex_allocation,
        }
    }

    pub fn draw(
        &mut self,
        device: &ash::Device,
        cmd: vk::CommandBuffer,
        screen_w: f32,
        screen_h: f32,
        requests: &[BlockIconRequest],
        uv_map: &crate::renderer::chunk::atlas::AtlasUVMap,
    ) {
        if requests.is_empty() {
            return;
        }

        unsafe {
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.atlas_set],
                &[],
            );
            device.cmd_bind_vertex_buffers(cmd, 0, &[self.vertex_buffer], &[0]);
        }

        let mut offset = 0usize;
        let mapped = self.vertex_allocation.mapped_slice_mut().unwrap();

        for req in requests {
            let mut verts: Vec<ChunkVertex> = Vec::new();

            for quad in req.quads {
                let light = gui_face_light(quad.cullface);
                let region = uv_map.get_region(&quad.texture);
                let u_span = region.u_max - region.u_min;
                let v_span = region.v_max - region.v_min;
                let tint = if quad.tinted {
                    [0.569, 0.741, 0.349]
                } else {
                    [1.0, 1.0, 1.0]
                };

                for i in [0, 1, 2, 2, 3, 0] {
                    verts.push(ChunkVertex {
                        position: quad.positions[i],
                        tex_coords: [
                            region.u_min + quad.uvs[i][0] * u_span,
                            region.v_min + quad.uvs[i][1] * v_span,
                        ],
                        light,
                        tint,
                    });
                }
            }

            if verts.is_empty() || offset + verts.len() > MAX_ICON_VERTICES {
                continue;
            }

            let bytes = bytemuck::cast_slice(&verts);
            let byte_offset = offset * VERTEX_SIZE;
            mapped[byte_offset..byte_offset + bytes.len()].copy_from_slice(bytes);

            let mvp = build_icon_mvp(req.screen_x, req.screen_y, req.size, screen_w, screen_h);
            let mvp_data = mvp.to_cols_array_2d();
            let mvp_bytes = bytemuck::bytes_of(&mvp_data);

            unsafe {
                device.cmd_push_constants(
                    cmd,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    mvp_bytes,
                );
                device.cmd_draw(cmd, verts.len() as u32, 1, offset as u32, 0);
            }

            offset += verts.len();
        }
    }

    pub fn recreate_pipeline(&mut self, device: &ash::Device, render_pass: vk::RenderPass) {
        unsafe { device.destroy_pipeline(self.pipeline, None) };
        self.pipeline = create_pipeline(device, render_pass, self.pipeline_layout);
    }

    pub fn destroy(&mut self, device: &ash::Device, allocator: &Arc<Mutex<Allocator>>) {
        unsafe {
            device.destroy_buffer(self.vertex_buffer, None);
        }
        allocator
            .lock()
            .unwrap()
            .free(std::mem::replace(&mut self.vertex_allocation, unsafe {
                std::mem::zeroed()
            }))
            .ok();
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_descriptor_set_layout(self.atlas_layout, None);
        }
    }
}

fn build_icon_mvp(px: f32, py: f32, size: f32, sw: f32, sh: f32) -> Mat4 {
    let model = Mat4::from_rotation_y(225.0f32.to_radians())
        * Mat4::from_rotation_x(30.0f32.to_radians())
        * Mat4::from_scale(Vec3::splat(0.625))
        * Mat4::from_translation(Vec3::new(-0.5, -0.5, -0.5));

    let mut ortho = Mat4::orthographic_rh(-0.6, 0.6, -0.6, 0.6, -5.0, 5.0);
    ortho.y_axis.y *= -1.0;

    let ndc_x = px / sw * 2.0 - 1.0;
    let ndc_y = py / sh * 2.0 - 1.0;
    let ndc_w = size / sw * 2.0;
    let ndc_h = size / sh * 2.0;

    let to_slot = Mat4::from_translation(Vec3::new(ndc_x + ndc_w * 0.5, ndc_y + ndc_h * 0.5, 0.0))
        * Mat4::from_scale(Vec3::new(ndc_w * 0.5, ndc_h * 0.5, 1.0));

    to_slot * ortho * model
}

fn gui_face_light(face: Option<Direction>) -> f32 {
    match face {
        Some(Direction::Up) => 0.98,
        Some(Direction::Down) => 0.50,
        Some(Direction::North) | Some(Direction::South) => 0.80,
        Some(Direction::East) | Some(Direction::West) => 0.608,
        None => 0.80,
    }
}

fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    layout: vk::PipelineLayout,
) -> vk::Pipeline {
    let vert_spv = shader::include_spirv!("block_icon.vert.spv");
    let frag_spv = shader::include_spirv!("block_icon.frag.spv");
    let vert_mod = shader::create_shader_module(device, vert_spv);
    let frag_mod = shader::create_shader_module(device, frag_spv);

    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_mod)
            .name(c"main"),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_mod)
            .name(c"main"),
    ];

    let binding = [vk::VertexInputBindingDescription {
        binding: 0,
        stride: VERTEX_SIZE as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }];
    let attrs = [
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 12,
        },
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R32_SFLOAT,
            offset: 20,
        },
        vk::VertexInputAttributeDescription {
            location: 3,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 24,
        },
    ];

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&binding)
        .vertex_attribute_descriptions(&attrs);
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .line_width(1.0);
    let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS);
    let blend_attachment = [vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::TRUE,
        src_color_blend_factor: vk::BlendFactor::ONE,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ONE,
        dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }];
    let color_blending =
        vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachment);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let info = [vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blending)
        .dynamic_state(&dynamic_state)
        .layout(layout)
        .render_pass(render_pass)
        .subpass(0)];

    let pipeline =
        unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &info, None) }
            .expect("failed to create block icon pipeline")[0];

    unsafe {
        device.destroy_shader_module(vert_mod, None);
        device.destroy_shader_module(frag_mod, None);
    }
    pipeline
}
