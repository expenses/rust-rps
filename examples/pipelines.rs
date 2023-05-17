use crate::reflection;

pub enum ShaderSource<'a> {
    Spirv(&'a str),
    Hlsl(&'a str, &'a str, &'a str),
}

impl<'a> ShaderSource<'a> {
    fn load(&self) -> Vec<u8> {
        match self {
            Self::Spirv(filename) => std::fs::read(filename).unwrap(),
            Self::Hlsl(filename, entry_name, profile) => {
                let text = std::fs::read_to_string(filename).unwrap();

                hassle_rs::compile_hlsl(filename, &text, entry_name, profile, &["-spirv"], &[])
                    .unwrap()
            }
        }
    }

    fn as_str(&self) -> &'a str {
        match self {
            Self::Spirv(filename) => filename,
            Self::Hlsl(filename, ..) => filename,
        }
    }
}

pub struct ComputePipeline {
    pub bind_group_layouts: std::collections::BTreeMap<u32, wgpu::BindGroupLayout>,
    pub pipeline: wgpu::ComputePipeline,
}

impl ComputePipeline {
    pub fn new(device: &wgpu::Device, shader: &ShaderSource, reflection_settings: &reflection::ReflectionSettings) -> Self {
        let shader_bytes = shader.load();

        //let reflected_entry_points = spirq::ReflectConfig::new().spv(&shader_bytes[..]).reflect().unwrap();
        //let entry_point = &reflected_entry_points[0];

        let reflection = reflection::reflect(&shader_bytes, reflection_settings);

        assert_eq!(reflection.entry_points.len(), 1);

        let entry_name = &reflection.entry_points[0].name;

        let mut bind_group_layouts = std::collections::BTreeMap::new();

        for (id, entries) in reflection.bindings.iter() {
            bind_group_layouts.insert(
                *id,
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &entries.values().cloned().collect::<Vec<_>>(),
                }),
            );
        }

        let bind_group_layout_refs: Vec<_> = bind_group_layouts.values().collect();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layout_refs,
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..reflection.max_push_constant_size as u32,
            }],
        });

        Self {
            bind_group_layouts,
            pipeline: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(shader.as_str()),
                layout: Some(&pipeline_layout),
                module: &unsafe {device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                    label: None,
                    source: wgpu::util::make_spirv_raw(&shader_bytes),
                })},
                entry_point: entry_name,
            }),
        }
    }
}

pub struct RenderPipeline {
    pub bind_group_layouts: std::collections::BTreeMap<u32, wgpu::BindGroupLayout>,
    pub pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        vertex_shader: &ShaderSource,
        fragment_shader: &ShaderSource,
        vertex_buffers: &[wgpu::VertexBufferLayout],
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Self {
        let vertex_shader_bytes = vertex_shader.load();
        let fragment_shader_bytes = fragment_shader.load();

        let vertex_reflection = 
        reflection::reflect(&vertex_shader_bytes, &Default::default());

        let vertex_entry_name = &vertex_reflection.entry_points[0].name;

        let fragment_reflection = 
        reflection::reflect(&fragment_shader_bytes, &Default::default());

        let fragment_entry_name = &fragment_reflection.entry_points[0].name;

        let bindings = reflection::merge_bind_group_layout_entries(&vertex_reflection.bindings, &fragment_reflection.bindings);
        
        let mut bind_group_layouts = std::collections::BTreeMap::new();

        for (id, entries) in bindings.iter() {
            bind_group_layouts.insert(
                *id,
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &entries.values().cloned().collect::<Vec<_>>(),
                }),
            );
        }

        let bind_group_layout_refs: Vec<_> = bind_group_layouts.values().collect();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layout_refs,
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                range: 0..vertex_reflection.max_push_constant_size.max(fragment_reflection.max_push_constant_size) as u32,
            }],
        });

        Self {
            bind_group_layouts,
            pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!(
                    "{} + {}",
                    vertex_entry_name, fragment_entry_name
                )),                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &unsafe {device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                        label: None,
                        source: wgpu::util::make_spirv_raw(&vertex_shader_bytes),
                    })},
                    entry_point: &vertex_entry_name,
                    buffers: vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &unsafe {device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                        label: None,
                        source: wgpu::util::make_spirv_raw(&fragment_shader_bytes),
                    })},
                    entry_point: &fragment_entry_name,
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
