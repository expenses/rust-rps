pub mod mapping;

pub use mapping::map_result;

pub mod sys {
    #![allow(rustdoc::broken_intra_doc_links)]
    #![allow(clippy::missing_safety_doc)]
    #![allow(clippy::useless_transmute)]
    #![allow(clippy::transmute_int_to_bool)]
    #![allow(clippy::too_many_arguments)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    pub use root::*;
}

type DynamicLibraryInitFunction =
    unsafe extern "C" fn(pProcs: *const sys::___rpsl_runtime_procs, sizeofProcs: u32) -> i32;

pub fn load_dynamic_rps_library(
    lib: &libloading::Library,
    entry_name: &str,
) -> Result<sys::RpsRpslEntry, sys::RpsResult> {
    let symbol: libloading::Symbol<DynamicLibraryInitFunction> =
        unsafe { lib.get(b"___rps_dyn_lib_init").unwrap() };

    map_result(unsafe { sys::rpsRpslDynamicLibraryInit(Some(*symbol.into_raw())) })?;

    let entry: libloading::Symbol<sys::RpsRpslEntry> =
        unsafe { lib.get(entry_name.as_bytes()) }.unwrap();

    Ok(unsafe { *(entry.into_raw().into_raw() as *const sys::RpsRpslEntry) })
}

pub fn bind_node_callback(
    subprogram: sys::RpsSubprogram,
    entry_point: &str,
    callback: sys::PFN_rpsCmdCallback,
) -> Result<(), sys::RpsResult> {
    let entry_point = std::ffi::CString::new(entry_point).unwrap();

    map_result(unsafe {
        sys::rpsProgramBindNodeCallback(
            subprogram,
            entry_point.as_ptr(),
            &sys::RpsCmdCallback {
                pfnCallback: callback,
                pUserContext: std::ptr::null_mut(),
                flags: 0,
            },
        )
    })
}

pub struct CmdCallbackContext<'a, Cmd, Usr> {
    pub inner: sys::rps::RuntimeCmdCallbackContext,
    pub command_buffer: &'a mut Cmd,
    pub user_data: &'a mut Usr,
    pub resources: &'a [sys::rps::ResourceInstance],
    pub args: &'a [*mut std::ffi::c_void],
}

impl<'a, Cmd, Usr> CmdCallbackContext<'a, Cmd, Usr> {
    pub unsafe fn new(context: *const sys::RpsCmdCallbackContext) -> Self {
        let context = *(context as *const sys::rps::RuntimeCmdCallbackContext);
        let resource_cache = &(*context.pRenderGraph).m_resourceCache;
        let resources = mapping::vector_to_slice(resource_cache);

        let base_context = context._base;

        let user_data = unsafe { &mut *(base_context.pUserRecordContext as *mut Usr) };

        let command_buffer = unsafe { &mut *(base_context.hCommandBuffer.ptr as *mut Cmd) };

        let args = std::slice::from_raw_parts(base_context.ppArgs, base_context.numArgs as usize);

        Self {
            inner: context,
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

pub struct Device {
    inner: sys::RpsDevice,
    create_info: sys::RpsDeviceCreateInfo,
}

impl Device {
    pub fn new() -> Result<Self, sys::RpsResult> {
        let create_info: sys::RpsDeviceCreateInfo = unsafe { std::mem::zeroed() };
        let mut rps_device: sys::RpsDevice = std::ptr::null_mut();
        map_result(unsafe { sys::rpsDeviceCreate(&create_info, &mut rps_device) })?;

        Ok(Self {
            inner: rps_device,
            create_info,
        })
    }

    pub fn add_callback_runtime<T>(
        &mut self,
        callbacks: sys::Callbacks,
        user_data: *mut T,
    ) -> Result<(), sys::RpsResult> {
        map_result(unsafe {
            sys::add_callback_runtime(
                &self.create_info,
                &mut self.inner,
                callbacks,
                user_data as _,
            )
        })
    }

    pub fn create_render_graph(
        &self,
        entry_point: sys::RpsRpslEntry,
    ) -> Result<RenderGraph, sys::RpsResult> {
        let mut graph: sys::RpsRenderGraph = std::ptr::null_mut();

        let mut graph_create_info: sys::RpsRenderGraphCreateInfo = unsafe { std::mem::zeroed() };

        let queue_flags: &[u32] = &[
            sys::RpsQueueFlagBits::RPS_QUEUE_FLAG_GRAPHICS as u32,
            sys::RpsQueueFlagBits::RPS_QUEUE_FLAG_COMPUTE as u32,
            sys::RpsQueueFlagBits::RPS_QUEUE_FLAG_COPY as u32,
        ];

        graph_create_info.scheduleInfo.pQueueInfos = queue_flags.as_ptr();
        graph_create_info.scheduleInfo.numQueues = queue_flags.len() as u32;
        graph_create_info.mainEntryCreateInfo.hRpslEntryPoint = entry_point;

        map_result(unsafe {
            sys::rpsRenderGraphCreate(self.inner, &graph_create_info, &mut graph)
        })?;

        Ok(RenderGraph { inner: graph })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { sys::rpsDeviceDestroy(self.inner) }
    }
}

pub struct RenderGraph {
    inner: sys::RpsRenderGraph,
}

impl RenderGraph {
    pub fn get_main_entry(&self) -> sys::RpsSubprogram {
        unsafe { sys::rpsRenderGraphGetMainEntry(self.inner) }
    }

    pub fn update(
        &self,
        update_info: &sys::RpsRenderGraphUpdateInfo,
    ) -> Result<(), sys::RpsResult> {
        map_result(unsafe { sys::rpsRenderGraphUpdate(self.inner, update_info) })
    }

    pub fn get_batch_layout(&self) -> Result<BatchLayout, sys::RpsResult> {
        let mut layout: sys::RpsRenderGraphBatchLayout = unsafe { std::mem::zeroed() };

        map_result(unsafe { sys::rpsRenderGraphGetBatchLayout(self.inner, &mut layout) })?;

        Ok(BatchLayout { inner: layout })
    }

    pub fn record_commands<Usr, Cmd>(
        &self,
        frame_index: u64,
        batch: sys::RpsCommandBatch,
        user_data: *mut Usr,
        command_buffer: &mut Cmd,
    ) -> Result<(), sys::RpsResult> {
        let record_info = sys::RpsRenderGraphRecordCommandInfo {
            cmdBeginIndex: batch.cmdBegin,
            numCmds: batch.numCmds,
            frameIndex: frame_index,
            flags: 0,
            hCmdBuffer: sys::RpsRuntimeCommandBuffer_T {
                ptr: command_buffer as *mut Cmd as _,
            },
            pUserContext: user_data as _,
        };

        map_result(unsafe { sys::rpsRenderGraphRecordCommands(self.inner, &record_info) })?;

        Ok(())
    }
}

pub struct BatchLayout {
    inner: sys::RpsRenderGraphBatchLayout,
}

impl BatchLayout {
    pub fn command_batches(&self) -> &[sys::RpsCommandBatch] {
        unsafe {
            std::slice::from_raw_parts(self.inner.pCmdBatches, self.inner.numCmdBatches as usize)
        }
    }
}
