use crate::reflection;

pub struct RenderPipeline {
    pub bind_group_layouts: std::collections::BTreeMap<u32, wgpu::BindGroupLayout>,
    pub pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        vertex_shader_filename: &str,
        fragment_shader_filename: &str,
        vertex_buffers: &[wgpu::VertexBufferLayout],
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Self {
        let vert_shader_bytes = std::fs::read(vertex_shader_filename).unwrap();
        let frag_shader_bytes = std::fs::read(fragment_shader_filename).unwrap();

        let settings = reflection::ShaderSettings::default();

        let vert_reflection =
            rspirv_reflect::Reflection::new_from_spirv(&vert_shader_bytes).unwrap();

        let (vert_bgl_entries, vert_entry_name) =
            reflection::reflect_bind_group_layout_entries(&vert_reflection, &settings);

        let frag_reflection =
            rspirv_reflect::Reflection::new_from_spirv(&frag_shader_bytes).unwrap();

        let (frag_bgl_entries, frag_entry_name) =
            reflection::reflect_bind_group_layout_entries(&frag_reflection, &settings);

        let (vert_push_constant_info, vert_push_constant_stages) =
            match vert_reflection.get_push_constant_range() {
                Ok(Some(info)) => (info, wgpu::ShaderStages::VERTEX),
                _ => (
                    rspirv_reflect::PushConstantInfo { offset: 0, size: 0 },
                    wgpu::ShaderStages::NONE,
                ),
            };
        let (frag_push_constant_info, frag_push_constant_stages) =
            match frag_reflection.get_push_constant_range() {
                Ok(Some(info)) => (info, wgpu::ShaderStages::FRAGMENT),
                _ => (
                    rspirv_reflect::PushConstantInfo { offset: 0, size: 0 },
                    wgpu::ShaderStages::NONE,
                ),
            };

        let push_constant_size = vert_push_constant_info
            .size
            .max(frag_push_constant_info.size);

        let merged_bgl_entries =
            reflection::merge_bind_group_layout_entries(&vert_bgl_entries, &frag_bgl_entries);

        let mut bind_group_layouts = std::collections::BTreeMap::new();

        for (id, entries) in merged_bgl_entries.iter() {
            bind_group_layouts.insert(
                *id,
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: entries,
                }),
            );
        }

        let bind_group_layout_refs: Vec<_> = bind_group_layouts.values().collect();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layout_refs,
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: vert_push_constant_stages | frag_push_constant_stages,
                range: 0..push_constant_size,
            }],
        });

        Self {
            bind_group_layouts,
            pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!(
                    "{} + {}",
                    vertex_shader_filename, fragment_shader_filename
                )),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::util::make_spirv(&vert_shader_bytes),
                    }),
                    entry_point: vert_entry_name,
                    buffers: vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::util::make_spirv(&frag_shader_bytes),
                    }),
                    entry_point: frag_entry_name,
                    targets,
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            }),
        }
    }
}
