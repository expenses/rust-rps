pub mod ffi {
    #![allow(clippy::useless_transmute)]
    #![allow(clippy::transmute_int_to_bool)]
    #![allow(clippy::too_many_arguments)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    pub use root::rps as cpp;

    pub use root::*;
}

pub use render_pipeline_shaders as rps;

use std::ffi::c_void;

pub struct CmdCallbackContext<'a, Cmd, Usr> {
    pub command_buffer: &'a mut Cmd,
    pub user_data: &'a mut Usr,
    pub resources: &'a [ffi::cpp::ResourceInstance],
    pub args: &'a [*mut c_void],
}

impl<Cmd, Usr> CmdCallbackContext<'_, Cmd, Usr> {
    pub unsafe fn new(context: *const rps::CmdCallbackContext) -> Self {
        let base_context = *context;

        let context = &*(context as *const ffi::cpp::RuntimeCmdCallbackContext);
        let resource_cache = &(*context.pRenderGraph).m_resourceCache;
        let resources = vector_to_slice(resource_cache);

        let user_data = unsafe { &mut *(base_context.user_record_context as *mut Usr) };

        let command_buffer = unsafe { &mut *(base_context.command_buffer.into_raw() as *mut Cmd) };

        let args = std::slice::from_raw_parts(base_context.args, base_context.num_args as usize);

        Self {
            command_buffer,
            user_data,
            resources,
            args,
        }
    }

    pub unsafe fn reinterpret_arg_as<T>(&self, index: usize) -> &T {
        &*(self.args[index] as *const T)
    }
}

pub fn add_callback_runtime<T>(
    device: &rps::Device,
    device_create_info: &rps::DeviceCreateInfo,
    callbacks: ffi::Callbacks,
    user_data: *mut T,
) -> rps::RpsResult<()> {
    unsafe {
        rps::result_from_ffi(ffi::add_callback_runtime(
            device_create_info as *const rps::DeviceCreateInfo as _,
            device as *const rps::Device as _,
            callbacks,
            user_data as _,
        ))
    }
}

pub fn vector_to_slice<T, A>(vector: &ffi::cpp::Vector<T, A>) -> &[T] {
    unsafe { std::slice::from_raw_parts(vector.m_pArray, vector.m_Count) }
}

pub fn array_ref_to_mut_slice<T>(array_ref: &mut ffi::cpp::ArrayRef<T, u64>) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(array_ref.m_pData, array_ref.m_Size as usize) }
}
