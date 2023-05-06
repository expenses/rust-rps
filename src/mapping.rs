use crate::rps;
use bitflags::bitflags;

pub fn map_result(rps_result: rps::RpsResult) -> Result<(), rps::RpsResult> {
    match rps_result {
        rps::RpsResult::RPS_OK => Ok(()),
        _ => Err(rps_result),
    }
}

pub fn map_wgpu_format_to_rps(format: wgpu::TextureFormat) -> rps::RpsFormat {
    match format {
        wgpu::TextureFormat::Bgra8UnormSrgb => rps::RpsFormat::RPS_FORMAT_B8G8R8A8_UNORM_SRGB,
        other => panic!("{:?}", other),
    }
}

pub fn map_rps_format_to_wgpu(format: rps::RpsFormat) -> wgpu::TextureFormat {
    match format {
        rps::RpsFormat::RPS_FORMAT_B8G8R8A8_UNORM_SRGB => wgpu::TextureFormat::Bgra8UnormSrgb,
        other => panic!("{:?}", other),
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ShaderStages: u32 {
        const VS = rps::root::RpsShaderStageBits::RPS_SHADER_STAGE_VS as u32;
        const PS = rps::root::RpsShaderStageBits::RPS_SHADER_STAGE_PS as u32;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct AccessFlagBits: i32 {
        const INDIRECT_ARGS = rps::root::RpsAccessFlagBits::RPS_ACCESS_INDIRECT_ARGS_BIT as i32;
        const INDEX_BUFFER = rps::root::RpsAccessFlagBits::RPS_ACCESS_INDEX_BUFFER_BIT as i32;
        const VERTEX_BUFFER = rps::root::RpsAccessFlagBits::RPS_ACCESS_VERTEX_BUFFER_BIT as i32;
        const CONSTANT_BUFFER = rps::root::RpsAccessFlagBits::RPS_ACCESS_CONSTANT_BUFFER_BIT as i32;
        const SHADER_RESOURCE = rps::root::RpsAccessFlagBits::RPS_ACCESS_SHADER_RESOURCE_BIT as i32;
        const UNORDERED_ACCESS = rps::root::RpsAccessFlagBits::RPS_ACCESS_UNORDERED_ACCESS_BIT as i32;
        const SHADING_RATE = rps::root::RpsAccessFlagBits::RPS_ACCESS_SHADING_RATE_BIT as i32;
        const RENDER_TARGET = rps::root::RpsAccessFlagBits::RPS_ACCESS_RENDER_TARGET_BIT as i32;
        const DEPTH_READ = rps::root::RpsAccessFlagBits::RPS_ACCESS_DEPTH_READ_BIT as i32;
        const DEPTH_WRITE = rps::root::RpsAccessFlagBits::RPS_ACCESS_DEPTH_WRITE_BIT as i32;
        const STENCIL_READ = rps::root::RpsAccessFlagBits::RPS_ACCESS_STENCIL_READ_BIT as i32;
        const STENCIL_WRITE = rps::root::RpsAccessFlagBits::RPS_ACCESS_STENCIL_WRITE_BIT as i32;
        const STREAM_OUT = rps::root::RpsAccessFlagBits::RPS_ACCESS_STREAM_OUT_BIT as i32;
        const COPY_SRC = rps::root::RpsAccessFlagBits::RPS_ACCESS_COPY_SRC_BIT as i32;
        const COPY_DEST = rps::root::RpsAccessFlagBits::RPS_ACCESS_COPY_DEST_BIT as i32;
        const RESOLVE_SRC = rps::root::RpsAccessFlagBits::RPS_ACCESS_RESOLVE_SRC_BIT as i32;
        const RESOLVE_DEST = rps::root::RpsAccessFlagBits::RPS_ACCESS_RESOLVE_DEST_BIT as i32;
        const RAYTRACING_AS_BUILD = rps::root::RpsAccessFlagBits::RPS_ACCESS_RAYTRACING_AS_BUILD_BIT as i32;
        const RAYTRACING_AS_READ = rps::root::RpsAccessFlagBits::RPS_ACCESS_RAYTRACING_AS_READ_BIT as i32;
        const PRESENT = rps::root::RpsAccessFlagBits::RPS_ACCESS_PRESENT_BIT as i32;
        const CPU_READ = rps::root::RpsAccessFlagBits::RPS_ACCESS_CPU_READ_BIT as i32;
        const CPU_WRITE = rps::root::RpsAccessFlagBits::RPS_ACCESS_CPU_WRITE_BIT as i32;
        const DISCARD_DATA_BEFORE = rps::root::RpsAccessFlagBits::RPS_ACCESS_DISCARD_DATA_BEFORE_BIT as i32;
        const DISCARD_DATA_AFTER = rps::root::RpsAccessFlagBits::RPS_ACCESS_DISCARD_DATA_AFTER_BIT as i32;
        const STENCIL_DISCARD_DATA_BEFORE = rps::root::RpsAccessFlagBits::RPS_ACCESS_STENCIL_DISCARD_DATA_BEFORE_BIT as i32;
        const STENCIL_DISCARD_DATA_AFTER = rps::root::RpsAccessFlagBits::RPS_ACCESS_STENCIL_DISCARD_DATA_AFTER_BIT as i32;
        const BEFORE = rps::root::RpsAccessFlagBits::RPS_ACCESS_BEFORE_BIT as i32;
        const AFTER = rps::root::RpsAccessFlagBits::RPS_ACCESS_AFTER_BIT as i32;
        const CLEAR = rps::root::RpsAccessFlagBits::RPS_ACCESS_CLEAR_BIT as i32;
    }
}

pub fn vector_to_slice<T, A>(vector: &rps::rps::Vector<T, A>) -> &[T] {
    unsafe { std::slice::from_raw_parts(vector.m_pArray, vector.m_Count) }
}

pub fn array_ref_to_mut_slice<T>(array_ref: &mut rps::rps::ArrayRef<T, u64>) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(array_ref.m_pData, array_ref.m_Size as usize) }
}
