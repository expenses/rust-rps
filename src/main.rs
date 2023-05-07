#![allow(nonstandard_style)]

use std::ffi::c_void;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

mod builtin_callbacks;
mod mapping;
mod node_callbacks;
mod reflection;

use mapping::{map_result, map_wgpu_format_to_rps};

pub mod rps {
    #![allow(rustdoc::broken_intra_doc_links)]
    #![allow(clippy::missing_safety_doc)]
    #![allow(clippy::useless_transmute)]
    #![allow(clippy::transmute_int_to_bool)]
    #![allow(clippy::too_many_arguments)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    pub use root::*;
}

type DynamicLibraryInitFunction =
    unsafe extern "C" fn(pProcs: *const rps::___rpsl_runtime_procs, sizeofProcs: u32) -> i32;

struct RenderPipeline {
    bind_group_layouts: std::collections::BTreeMap<u32, wgpu::BindGroupLayout>,
    pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    fn new(
        device: &wgpu::Device,
        vertex_shader_filename: &str,
        fragment_shader_filename: &str,
        format: wgpu::TextureFormat,
    ) -> Self {
        let vert_shader_bytes = std::fs::read(vertex_shader_filename).unwrap();
        let frag_shader_bytes = std::fs::read(fragment_shader_filename).unwrap();

        let settings = reflection::ShaderSettings::default();

        let vert_reflection =
            rspirv_reflect::Reflection::new_from_spirv(&vert_shader_bytes).unwrap();

        let vert_bgl_entries =
            reflection::reflect_bind_group_layout_entries(&vert_reflection, &settings);

        let frag_reflection =
            rspirv_reflect::Reflection::new_from_spirv(&frag_shader_bytes).unwrap();

        let frag_bgl_entries =
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
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::util::make_spirv(&vert_shader_bytes),
                    }),
                    entry_point: "VSMain",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::util::make_spirv(&frag_shader_bytes),
                    }),
                    entry_point: "PSMain",
                    targets: &[Some(format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            }),
        }
    }
}

struct UserData {
    triangle_pipeline: RenderPipeline,
    multithreaded_triangle_pipeline: RenderPipeline,
    blit_pipeline: RenderPipeline,
    device: wgpu::Device,
    sampler: wgpu::Sampler,
}

struct CommandBuffer {
    encoder: Option<wgpu::CommandEncoder>,
}

#[derive(StructOpt)]
struct Opts {
    filename: PathBuf,
    entry_point: String,
}

use wasmtime::*;

fn offset_pointer_by_pointer<A, B>(pointer: *const A, offset: *const B) -> *const A {
    unsafe { (pointer as *const u8).add(offset as usize) as *const A }
}

type StoreInner = usize;

struct WasmEntry {
    func: TypedFunc<(u32, u64, u32), ()>,
    store: Store<StoreInner>,
    memory: Memory,
    params: Vec<rps::RpsParameterDesc>,
}

static mut WASM_ENTRY: Option<WasmEntry> = None;

unsafe extern "C" fn wasm_entry_point(
    numArgs: u32,
    mut ppArgs: *const *const c_void,
    flags: rps::RpslEntryCallFlags,
) {
    let entry = WASM_ENTRY.as_mut().unwrap();

    let mut offset = 67904; //;entry.memory.data_size(&entry.store);

    let ptr = offset as u64;

    let x = (*ppArgs) as *const rps::RpsResourceDesc;
    let x = *x;
    let img = x.__bindgen_anon_1.image;
    dbg!(x, entry.params[0], img.width, img.height, img.__bindgen_anon_1.depth, );

    let args = std::slice::from_raw_parts(ppArgs, numArgs as usize);

    for (i, &arg) in args.into_iter().enumerate() {
        let arg_slice =
            std::slice::from_raw_parts(arg as *const u8, entry.params[i].typeInfo.size as usize);

        entry
            .memory
            .write(&mut entry.store, offset, arg_slice)
            .unwrap();

        offset += arg_slice.len();
    }

    entry
        .memory
        .write(&mut entry.store, offset, &ptr.to_le_bytes())
        .unwrap();

    let ptr_ptr = offset as u64;

    entry
        .func
        .call(&mut entry.store, (numArgs, ptr_ptr, flags))
        .unwrap()
}

fn main() {
    let opts = Opts::from_args();

    let file_stem = opts.filename.file_stem().unwrap().to_str().unwrap();

    let entry_name = format!("rpsl_M_{}_E_{}", file_stem, opts.entry_point);

    let engine = Engine::new(Config::new().wasm_memory64(true)).unwrap();
    let mut store = Store::new(&engine, 0);
    let module = Module::from_file(&engine, &opts.filename).unwrap();

    let rpsl_abort = Func::wrap(
        &mut store,
        |caller: Caller<'_, StoreInner>, error_code: i32| {
            println!("got error code: {}", error_code);
        },
    );

    let rpsl_block_marker = Func::wrap(
        &mut store,
        |caller: Caller<'_, StoreInner>,
         marker_type: u32,
         block_index: u32,
         resource_count: u32,
         node_count: u32,
         local_loop_index: u32,
         num_children: u32,
         parent_id: u32| {
            let err = map_result(unsafe {
                rps::RpslHostBlockMarker(
                    marker_type,
                    block_index,
                    resource_count,
                    node_count,
                    local_loop_index,
                    num_children,
                    parent_id,
                )
            });

            err.unwrap();
        },
    );

    let rpsl_describe_handle = Func::wrap(
        &mut store,
        |caller: Caller<'_, StoreInner>,
         mut pOutData: /* *mut c_void*/ i64,
         dataSize: u32,
         mut inHandle: /* *const u32*/ i64,
         describeOp: u32| {
            let offset = *caller.data();
            pOutData += offset as i64;
            inHandle += offset as i64;

            let err = map_result(unsafe {
                rps::RpslHostDescribeHandle(pOutData as _, dataSize, inHandle as _, describeOp)
            })
            .unwrap();
        },
    );

    let rpsl_dxop_binary_i32 = Func::wrap(
        &mut store,
        |caller: Caller<'_, StoreInner>, op: u32, a: u32, b: u32| -> u32 {
            fn to_signed(value: u32) -> i32 {
                i32::from_le_bytes(value.to_le_bytes())
            }

            fn to_unsigned(value: i32) -> u32 {
                u32::from_le_bytes(value.to_le_bytes())
            }

            match op {
                rps::DXILOpCode_IMax => to_unsigned(to_signed(a).max(to_signed(b))),
                rps::DXILOpCode_IMin => to_unsigned(to_signed(a).min(to_signed(b))),
                rps::DXILOpCode_UMax => a.max(b),
                rps::DXILOpCode_UMin => a.min(b),
                _ => unimplemented!("{}", op),
            }
        },
    );

    let rpsl_create_resource = Func::wrap(
        &mut store,
        |caller: Caller<'_, StoreInner>,
         ty: u32,
         flags: u32,
         format: u32,
         width: u32,
         height: u32,
         depthOrArraySize: u32,
         mipLevels: u32,
         sampleCount: u32,
         sampleQuality: u32,
         temporalLayers: u32,
         id: u32| {
            let mut resource_id = 0;

            dbg!(depthOrArraySize, mipLevels);

            map_result(unsafe {
                rps::RpslHostCreateResource(
                    ty,
                    flags,
                    format,
                    width,
                    height,
                    depthOrArraySize,
                    mipLevels,
                    sampleCount,
                    sampleQuality,
                    temporalLayers,
                    id,
                    &mut resource_id,
                )
            })
            .unwrap();

            resource_id
        },
    );

    let rpsl_name_resource = Func::wrap(
        &mut store,
        |caller: Caller<'_, StoreInner>, resource_hdl: u32, mut name: i64, name_length: u32| {

            let offset = *caller.data();
            name += offset as i64;
            let err = map_result(unsafe {
                rps::RpslHostNameResource(resource_hdl, name as _, name_length)
            });

            err.unwrap();
        },
    );

    let rpsl_node_call = Func::wrap(
        &mut store,
        |caller: Caller<'_, StoreInner>,
         nodeDeclId: u32,
         numArgs: u32,
         mut ppArgs: i64,
         nodeCallFlags: u32,
         nodeId: u32| {
            let offset = *caller.data();

            ppArgs += offset as i64;

            unsafe {
                for i in 0..numArgs {
                    let mut ptr = (ppArgs as *mut *const c_void).add(i as usize);
                    *ptr = (*ptr).add(offset as usize);
                }
            }

            let mut cmdid = 0;
            let err = map_result(unsafe {
                rps::RpslHostCallNode(
                    nodeDeclId,
                    numArgs,
                    ppArgs as _,
                    nodeCallFlags,
                    nodeId,
                    &mut cmdid,
                )
            })
            .unwrap();

            cmdid
        },
    );

    let mut instance_imports = Vec::new();

    for import in module.imports() {
        match import.name() {
            "___rpsl_describe_handle" => instance_imports.push(rpsl_describe_handle.into()),
            "___rpsl_abort" => instance_imports.push(rpsl_abort.into()),
            "___rpsl_block_marker" => instance_imports.push(rpsl_block_marker.into()),
            "___rpsl_node_call" => instance_imports.push(rpsl_node_call.into()),
            "___rpsl_dxop_binary_i32" => instance_imports.push(rpsl_dxop_binary_i32.into()),
            "___rpsl_create_resource" => instance_imports.push(rpsl_create_resource.into()),
            "___rpsl_name_resource" => instance_imports.push(rpsl_name_resource.into()),
            other => panic!("Import not handled: {}", other),
        }
    }

    let instance = Instance::new(
        &mut store,
        &module,
        &instance_imports,
    )
    .unwrap();

    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let entry_value_pointer = instance
        .get_global(&mut store, &format!("{}_AE_value", entry_name))
        .unwrap()
        .get(&mut store)
        .unwrap_i64();

    let x = {
        let data = memory.data(&store);
        let offset = &data[0] as *const u8;
        offset as usize
    };

    *store.data_mut() = x;

    let entry = unsafe {
        let data = memory.data(&store);

        let offset = &data[0] as *const u8;
        let ptr = &data[entry_value_pointer as usize] as *const u8 as *mut rps::rps::RpslEntry;
        let mut actual = &mut *ptr;
        actual.pfnEntry = Some(wasm_entry_point);
        actual.name = offset_pointer_by_pointer(actual.name, offset);
        actual.pParamDescs = offset_pointer_by_pointer(actual.pParamDescs, offset);
        actual.pNodeDecls = offset_pointer_by_pointer(actual.pNodeDecls, offset);

        let params = std::slice::from_raw_parts_mut(
            actual.pParamDescs as *mut rps::RpsParameterDesc,
            actual.numParams as usize,
        );

        let nodes = std::slice::from_raw_parts_mut(
            actual.pNodeDecls as *mut rps::RpsNodeDesc,
            actual.numNodeDecls as usize,
        );

        for param in &mut *params {
            param.attr = offset_pointer_by_pointer(param.attr, offset);
            param.name = offset_pointer_by_pointer(param.name, offset);
        }

        for node in &mut *nodes {
            node.name = offset_pointer_by_pointer(node.name, offset);
            node.pParamDescs = offset_pointer_by_pointer(node.pParamDescs, offset);

            let params = std::slice::from_raw_parts_mut(
                node.pParamDescs as *mut rps::RpsParameterDesc,
                node.numParams as usize,
            );

            for param in &mut *params {
                param.attr = offset_pointer_by_pointer(param.attr, offset);
                param.name = offset_pointer_by_pointer(param.name, offset);
            }

            dbg!(node);
        }

        let wrapper_fn = instance
            .get_typed_func(
                &mut store,
                &format!("rpsl_M_{}_Fn_{}_wrapper", file_stem, opts.entry_point),
            )
            .unwrap();

        WASM_ENTRY = Some(WasmEntry {
            store,
            memory,
            func: wrapper_fn,
            params: params.to_vec(),
        });

        //let (store, memory) = WASM_ENTRY.as_mut().map(|entry| (&mut entry.store, &mut entry.memory)).unwrap();

        ptr
    };

    let start = std::time::Instant::now();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::PUSH_CONSTANTS,
            limits: wgpu::Limits {
                max_push_constant_size: 4,
                ..Default::default()
            },
            ..Default::default()
        },
        None,
    ))
    .unwrap();

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];
    let size = window.inner_size();

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    let user_data = Box::new(UserData {
        multithreaded_triangle_pipeline: RenderPipeline::new(
            &device,
            "shaders/triangle_breathing.vert.spv",
            "shaders/triangle_breathing.frag.spv",
            swapchain_format,
        ),
        triangle_pipeline: RenderPipeline::new(
            &device,
            "shaders/triangle.vert.spv",
            "shaders/triangle.frag.spv",
            swapchain_format,
        ),
        blit_pipeline: RenderPipeline::new(
            &device,
            "shaders/blit.vert.spv",
            "shaders/blit.frag.spv",
            swapchain_format,
        ),
        sampler: device.create_sampler(&Default::default()),
        device,
    });

    let user_data_raw = Box::into_raw(user_data);

    let create_info: rps::RpsDeviceCreateInfo = unsafe { std::mem::zeroed() };

    let mut rps_device: rps::RpsDevice = std::ptr::null_mut();

    map_result(unsafe { rps::rpsDeviceCreate(&create_info, &mut rps_device) }).unwrap();

    map_result(unsafe {
        rps::add_callback_runtime(
            &create_info,
            &mut rps_device,
            rps::Callbacks {
                create_command_resources: Some(builtin_callbacks::create_command_resources),
                clear_color: Some(builtin_callbacks::clear_color),
                create_resources: Some(builtin_callbacks::create_resources),
                destroy_runtime_resource_deferred: Some(
                    builtin_callbacks::destroy_runtime_resource_deferred,
                ),
            },
            user_data_raw as _,
        )
    })
    .unwrap();

    let mut graph_create_info: rps::RpsRenderGraphCreateInfo = unsafe { std::mem::zeroed() };

    let queue_flags: &[u32] = &[
        rps::RpsQueueFlagBits::RPS_QUEUE_FLAG_GRAPHICS as u32,
        rps::RpsQueueFlagBits::RPS_QUEUE_FLAG_COMPUTE as u32,
        rps::RpsQueueFlagBits::RPS_QUEUE_FLAG_COPY as u32,
    ];

    let mut graph: rps::RpsRenderGraph = std::ptr::null_mut();

    graph_create_info.scheduleInfo.pQueueInfos = queue_flags.as_ptr();
    graph_create_info.scheduleInfo.numQueues = queue_flags.len() as u32;
    graph_create_info.mainEntryCreateInfo.hRpslEntryPoint = (entry) as rps::RpsRpslEntry;

    map_result(unsafe { rps::rpsRenderGraphCreate(rps_device, &graph_create_info, &mut graph) })
        .unwrap();

    let subprogram = unsafe { rps::rpsRenderGraphGetMainEntry(graph) };

    if let Err(err) = map_result(unsafe {
        rps::rpsProgramBindNodeCallback(
            subprogram,
            b"GeometryPass\0".as_ptr() as _,
            &rps::RpsCmdCallback {
                pfnCallback: Some(node_callbacks::geometry_pass),
                pUserContext: std::ptr::null_mut(),
                flags: 0,
            },
        )
    }) {
        eprintln!("Error binding GeometryPass node: {:?}", err);
    }
    if let Err(err) = map_result(unsafe {
        rps::rpsProgramBindNodeCallback(
            subprogram,
            b"Triangle\0".as_ptr() as _,
            &rps::RpsCmdCallback {
                pfnCallback: Some(node_callbacks::draw_triangle),
                pUserContext: std::ptr::null_mut(),
                flags: 0,
            },
        )
    }) {
        eprintln!("Error binding triangle node: {:?}", err);
    }

    if let Err(err) = map_result(unsafe {
        rps::rpsProgramBindNodeCallback(
            subprogram,
            b"Upscale\0".as_ptr() as _,
            &rps::RpsCmdCallback {
                pfnCallback: Some(node_callbacks::upscale),
                pUserContext: std::ptr::null_mut(),
                flags: 0,
            },
        )
    }) {
        eprintln!("Error binding upscale node: {:?}", err);
    };

    let mut completed_frame_index = u64::max_value();
    let mut frame_index = 0;
    let mut first_time = true;

    event_loop.run(move |event, _, control_flow| {
        let user_data = unsafe { &mut *user_data_raw };

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&user_data.device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                let mut update_info: rps::RpsRenderGraphUpdateInfo = unsafe { std::mem::zeroed() };

                let back_buffer = rps::RpsResourceDesc {
                    type_: rps::RpsResourceType::RPS_RESOURCE_TYPE_IMAGE_2D,
                    temporalLayers: 1,
                    flags: 0,
                    __bindgen_anon_1: rps::RpsResourceDesc__bindgen_ty_1 {
                        image: rps::RpsResourceDesc__bindgen_ty_1__bindgen_ty_1 {
                            width: config.width,
                            height: config.height,
                            mipLevels: 1,
                            sampleCount: 1,
                            format: map_wgpu_format_to_rps(swapchain_format),
                            __bindgen_anon_1:
                                rps::RpsResourceDesc__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1 {
                                    arrayLayers: 1,
                                },
                        },
                    },
                };

                let time = (std::time::Instant::now() - start).as_secs_f32();

                let args: &[rps::RpsConstant] = &[
                    (&back_buffer) as *const rps::RpsResourceDesc as _,
                    (&time) as *const f32 as _,
                ];

                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");

                let backbuffer_res = Box::new(Resource::SurfaceFrame(
                    frame.texture.create_view(&Default::default()),
                ));

                let res = rps::RpsRuntimeResource {
                    ptr: Box::into_raw(backbuffer_res) as *mut c_void,
                };

                let res_p = (&res) as *const rps::RpsRuntimeResource;

                let arg_resources: &[*const rps::RpsRuntimeResource] = &[res_p];

                update_info.frameIndex = frame_index;
                update_info.gpuCompletedFrameIndex = completed_frame_index;
                update_info.numArgs = args.len() as u32;
                update_info.ppArgs = args.as_ptr();
                update_info.ppArgResources = arg_resources.as_ptr();
                if first_time {
                    update_info.diagnosticFlags =
                        rps::RpsDiagnosticFlagBits::RPS_DIAGNOSTIC_ENABLE_ALL as u32;
                }

                first_time = false;

                map_result(unsafe { rps::rpsRenderGraphUpdate(graph, &update_info) }).unwrap();

                panic!();

                {
                    let mut layout: rps::RpsRenderGraphBatchLayout = unsafe { std::mem::zeroed() };

                    map_result(unsafe { rps::rpsRenderGraphGetBatchLayout(graph, &mut layout) })
                        .unwrap();

                    let batches = unsafe {
                        std::slice::from_raw_parts(
                            layout.pCmdBatches,
                            layout.numCmdBatches as usize,
                        )
                    };

                    for batch in batches {
                        let encoder = user_data.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor { label: None },
                        );

                        let mut cb = CommandBuffer {
                            encoder: Some(encoder),
                        };

                        let record_info = rps::RpsRenderGraphRecordCommandInfo {
                            cmdBeginIndex: batch.cmdBegin,
                            numCmds: batch.numCmds,
                            frameIndex: frame_index,
                            flags: 0,
                            hCmdBuffer: rps::RpsRuntimeCommandBuffer_T {
                                ptr: (&mut cb) as *mut CommandBuffer as _,
                            },
                            pUserContext: user_data_raw as _,
                        };

                        map_result(unsafe {
                            rps::rpsRenderGraphRecordCommands(graph, &record_info)
                        })
                        .unwrap();

                        let encoder = cb.encoder.take().unwrap();

                        queue.submit(Some(encoder.finish()));
                    }

                    completed_frame_index = frame_index;
                    frame_index += 1;
                }

                frame.present();
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            _ => {}
        }
    });
}

enum Resource {
    SurfaceFrame(wgpu::TextureView),
    Texture(wgpu::Texture),
}

enum BorrowedOrOwned<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<'a, T> std::ops::Deref for BorrowedOrOwned<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            Self::Owned(owned) => owned,
            Self::Borrowed(borrowed) => borrowed,
        }
    }
}
