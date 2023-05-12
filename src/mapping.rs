use crate::sys;
use bitflags::bitflags;

pub fn map_result(rps_result: sys::RpsResult) -> Result<(), sys::RpsResult> {
    match rps_result {
        sys::RpsResult::RPS_OK => Ok(()),
        _ => Err(rps_result),
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ShaderStages: u32 {
        const VS = sys::RpsShaderStageBits::RPS_SHADER_STAGE_VS as u32;
        const PS = sys::RpsShaderStageBits::RPS_SHADER_STAGE_PS as u32;
        const GS = sys::RpsShaderStageBits::RPS_SHADER_STAGE_GS as u32;
        const CS = sys::RpsShaderStageBits::RPS_SHADER_STAGE_CS as u32;
        const HS = sys::RpsShaderStageBits::RPS_SHADER_STAGE_HS as u32;
        const RAYTRACING = sys::RpsShaderStageBits::RPS_SHADER_STAGE_RAYTRACING as u32;
        const AS = sys::RpsShaderStageBits::RPS_SHADER_STAGE_AS as u32;
        const MS = sys::RpsShaderStageBits::RPS_SHADER_STAGE_MS as u32;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct AccessFlagBits: i32 {
        const INDIRECT_ARGS = sys::RpsAccessFlagBits::RPS_ACCESS_INDIRECT_ARGS_BIT as i32;
        const INDEX_BUFFER = sys::RpsAccessFlagBits::RPS_ACCESS_INDEX_BUFFER_BIT as i32;
        const VERTEX_BUFFER = sys::RpsAccessFlagBits::RPS_ACCESS_VERTEX_BUFFER_BIT as i32;
        const CONSTANT_BUFFER = sys::RpsAccessFlagBits::RPS_ACCESS_CONSTANT_BUFFER_BIT as i32;
        const SHADER_RESOURCE = sys::RpsAccessFlagBits::RPS_ACCESS_SHADER_RESOURCE_BIT as i32;
        const UNORDERED_ACCESS = sys::RpsAccessFlagBits::RPS_ACCESS_UNORDERED_ACCESS_BIT as i32;
        const SHADING_RATE = sys::RpsAccessFlagBits::RPS_ACCESS_SHADING_RATE_BIT as i32;
        const RENDER_TARGET = sys::RpsAccessFlagBits::RPS_ACCESS_RENDER_TARGET_BIT as i32;
        const DEPTH_READ = sys::RpsAccessFlagBits::RPS_ACCESS_DEPTH_READ_BIT as i32;
        const DEPTH_WRITE = sys::RpsAccessFlagBits::RPS_ACCESS_DEPTH_WRITE_BIT as i32;
        const STENCIL_READ = sys::RpsAccessFlagBits::RPS_ACCESS_STENCIL_READ_BIT as i32;
        const STENCIL_WRITE = sys::RpsAccessFlagBits::RPS_ACCESS_STENCIL_WRITE_BIT as i32;
        const STREAM_OUT = sys::RpsAccessFlagBits::RPS_ACCESS_STREAM_OUT_BIT as i32;
        const COPY_SRC = sys::RpsAccessFlagBits::RPS_ACCESS_COPY_SRC_BIT as i32;
        const COPY_DEST = sys::RpsAccessFlagBits::RPS_ACCESS_COPY_DEST_BIT as i32;
        const RESOLVE_SRC = sys::RpsAccessFlagBits::RPS_ACCESS_RESOLVE_SRC_BIT as i32;
        const RESOLVE_DEST = sys::RpsAccessFlagBits::RPS_ACCESS_RESOLVE_DEST_BIT as i32;
        const RAYTRACING_AS_BUILD = sys::RpsAccessFlagBits::RPS_ACCESS_RAYTRACING_AS_BUILD_BIT as i32;
        const RAYTRACING_AS_READ = sys::RpsAccessFlagBits::RPS_ACCESS_RAYTRACING_AS_READ_BIT as i32;
        const PRESENT = sys::RpsAccessFlagBits::RPS_ACCESS_PRESENT_BIT as i32;
        const CPU_READ = sys::RpsAccessFlagBits::RPS_ACCESS_CPU_READ_BIT as i32;
        const CPU_WRITE = sys::RpsAccessFlagBits::RPS_ACCESS_CPU_WRITE_BIT as i32;
        const DISCARD_DATA_BEFORE = sys::RpsAccessFlagBits::RPS_ACCESS_DISCARD_DATA_BEFORE_BIT as i32;
        const DISCARD_DATA_AFTER = sys::RpsAccessFlagBits::RPS_ACCESS_DISCARD_DATA_AFTER_BIT as i32;
        const STENCIL_DISCARD_DATA_BEFORE = sys::RpsAccessFlagBits::RPS_ACCESS_STENCIL_DISCARD_DATA_BEFORE_BIT as i32;
        const STENCIL_DISCARD_DATA_AFTER = sys::RpsAccessFlagBits::RPS_ACCESS_STENCIL_DISCARD_DATA_AFTER_BIT as i32;
        const BEFORE = sys::RpsAccessFlagBits::RPS_ACCESS_BEFORE_BIT as i32;
        const AFTER = sys::RpsAccessFlagBits::RPS_ACCESS_AFTER_BIT as i32;
        const CLEAR = sys::RpsAccessFlagBits::RPS_ACCESS_CLEAR_BIT as i32;
    }
}

pub fn vector_to_slice<T, A>(vector: &sys::rps::Vector<T, A>) -> &[T] {
    unsafe { std::slice::from_raw_parts(vector.m_pArray, vector.m_Count) }
}

pub fn array_ref_to_mut_slice<T>(array_ref: &mut sys::rps::ArrayRef<T, u64>) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(array_ref.m_pData, array_ref.m_Size as usize) }
}
