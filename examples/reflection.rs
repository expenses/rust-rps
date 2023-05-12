use std::collections::BTreeMap;

pub struct ShaderSettings {
    pub allow_texture_filtering: bool,
}

impl Default for ShaderSettings {
    fn default() -> Self {
        Self {
            allow_texture_filtering: true,
        }
    }
}

fn reflect_shader_stages(reflection: &rspirv_reflect::Reflection) -> (wgpu::ShaderStages, &str) {
    let entry_point_inst = &reflection.0.entry_points[0];

    let execution_model = entry_point_inst.operands[0].unwrap_execution_model();

    let entry_name = entry_point_inst.operands[2].unwrap_literal_string();

    let stages = match execution_model {
        rspirv_reflect::rspirv::spirv::ExecutionModel::Vertex => wgpu::ShaderStages::VERTEX,
        rspirv_reflect::rspirv::spirv::ExecutionModel::Fragment => wgpu::ShaderStages::FRAGMENT,
        rspirv_reflect::rspirv::spirv::ExecutionModel::GLCompute => wgpu::ShaderStages::COMPUTE,
        other => unimplemented!("{:?}", other),
    };

    (stages, entry_name)
}

pub fn reflect_bind_group_layout_entries<'a>(
    reflection: &'a rspirv_reflect::Reflection,
    settings: &ShaderSettings,
) -> (BTreeMap<u32, Vec<wgpu::BindGroupLayoutEntry>>, &'a str) {
    let (shader_stages, entry_name) = reflect_shader_stages(reflection);

    let descriptor_sets = reflection
        .get_descriptor_sets()
        .expect("Failed to get descriptor sets for shader reflection");

    let sets = descriptor_sets
        .iter()
        .map(|(location, set)| {
            let entries = set
                .iter()
                .map(|(&binding, info)| wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: shader_stages,
                    count: match info.binding_count {
                        rspirv_reflect::BindingCount::One => None,
                        rspirv_reflect::BindingCount::StaticSized(size) => {
                            Some(std::num::NonZeroU32::new(size as u32).expect("size is 0"))
                        }
                        rspirv_reflect::BindingCount::Unbounded => {
                            unimplemented!("No good way to handle unbounded binding counts yet.")
                        }
                    },
                    ty: match info.ty {
                        rspirv_reflect::DescriptorType::UNIFORM_BUFFER => {
                            wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            }
                        }
                        rspirv_reflect::DescriptorType::SAMPLER => {
                            wgpu::BindingType::Sampler(if settings.allow_texture_filtering {
                                wgpu::SamplerBindingType::Filtering
                            } else {
                                wgpu::SamplerBindingType::NonFiltering
                            })
                        }
                        rspirv_reflect::DescriptorType::SAMPLED_IMAGE => {
                            wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float {
                                    filterable: settings.allow_texture_filtering,
                                },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            }
                        }
                        rspirv_reflect::DescriptorType::STORAGE_BUFFER => {
                            wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            }
                        }
                        _ => unimplemented!("{:?}", info.ty),
                    },
                })
                .collect();

            (*location, entries)
        })
        .collect();

    (sets, entry_name)
}

pub fn merge_bind_group_layout_entries(
    a: &BTreeMap<u32, Vec<wgpu::BindGroupLayoutEntry>>,
    b: &BTreeMap<u32, Vec<wgpu::BindGroupLayoutEntry>>,
) -> BTreeMap<u32, Vec<wgpu::BindGroupLayoutEntry>> {
    let mut output = a.clone();

    for (location, entries) in b {
        let merged = output.entry(*location).or_default();

        for merging_entry in entries {
            if let Some(entry) = merged
                .iter_mut()
                .find(|entry| entry.binding == merging_entry.binding)
            {
                entry.visibility |= merging_entry.visibility;
            } else {
                merged.push(*merging_entry);
            }
        }
    }

    output
}
