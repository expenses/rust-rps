#![allow(nonstandard_style)]

pub mod rps;

type DynamicLibraryInitFunction =
    unsafe extern "C" fn(pProcs: *const rps::___rpsl_runtime_procs, sizeofProcs: u32) -> i32;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct RpslEntry {
    name: *const std::ffi::c_char,
    pfnEntry: rps::PFN_RpslEntry,
    pParamDescs: *const rps::RpsParameterDesc,
    pNodeDecls: *const rps::RpsNodeDesc,
    numParams: u32,
    numNodeDecls: u32,
}

extern "C" {
    #[no_mangle]
    static rpsl_M_hello_triangle_E_main: rps::RpsRpslEntry;
}

fn vector_to_slice<T, A>(vector: &rps::rps_Vector<T, A>) -> &[T] {
    unsafe { std::slice::from_raw_parts(vector.m_pArray, vector.m_Count) }
}

fn array_ref_to_slice<T>(array_ref: &rps::rps_ArrayRef<T, u64>) -> &[T] {
    unsafe { std::slice::from_raw_parts(array_ref.m_pData, array_ref.m_Size as usize) }
}

unsafe extern "C" fn create_command_resources(
    x: *const std::ffi::c_void,
    user_data: *mut std::ffi::c_void,
) -> rps::RpsResult {
    let user_data = &mut *(user_data as *mut UserData);
    //dbg!(&user_data);

    let x = x as *const rps::rps_RenderGraphUpdateContext;

    //println!("Hello from init");
    //dbg!(*x, *(*x).pUpdateInfo);

    let render_graph = &*(*x).renderGraph;

    let commands = vector_to_slice(&render_graph.m_cmds);
    let runtime_commands = vector_to_slice(&render_graph.m_runtimeCmdInfos);

    for batch in vector_to_slice(&render_graph.m_cmdBatches) {
        for i in batch.cmdBegin..batch.cmdBegin + batch.numCmds {
            let command = runtime_commands[i as usize];

            if command.isTransition() == 1 {
                continue;
            }

            let command = commands[command.cmdId() as usize];
            let node_decl = *command.pNodeDecl;
            let cmd = *command.pCmdDecl;
            let render_pass_info = *node_decl.pRenderPassInfo;
            //let render_pass_info = std::mem::transmute::<_, NodeDeclRenderPassInfo>(render_pass_info);

            let args = array_ref_to_slice(&cmd.args);

            if render_pass_info.clearOnly() == 1 {
                let refs = /*render_pass_info.render_target_clear_value_refs();*/

                {
                    std::slice::from_raw_parts(
                        render_pass_info.paramRefs.add(render_pass_info.clearValueRefs() as usize),
                        render_pass_info.renderTargetClearMask().count_ones() as usize
                    )
                };

                for r in refs {
                    let clear_colour_ptr =
                        args[r.paramId as usize] as *const rps::RpsClearColorValue;
                    //dbg!((*clear_colour_ptr).float32);
                    user_data.clear_colour = (*clear_colour_ptr).float32;
                }
            }
        }
    }

    rps::RpsResult::RPS_OK
}
extern "C" fn draw_triangle_cb(context: *const rps::RpsCmdCallbackContext) {
    let context = unsafe { *context };
    let user_data = unsafe{&*(context.pUserRecordContext as *mut UserData)};

    let render_pass = unsafe {
        &mut *(context.hCommandBuffer.ptr as *mut wgpu::RenderPass)
    };

    render_pass.set_pipeline(&user_data.triangle_pipeline);
    render_pass.draw(0..3, 0..1);
}

#[derive(Debug)]
struct UserData {
    clear_colour: [f32; 4],
    triangle_pipeline: wgpu::RenderPipeline,
}

fn main() {

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

    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
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

    let vert_shader = device.create_shader_module(wgpu::include_spirv!("../triangle.vert.spv"));
    let frag_shader = device.create_shader_module(wgpu::include_spirv!("../triangle.frag.spv"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vert_shader,
            entry_point: "VSMain",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &frag_shader,
            entry_point: "PSMain",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut user_data = Box::new(UserData {
        clear_colour: [1.0; 4],
        triangle_pipeline: render_pipeline
    });

    let user_data_raw = Box::into_raw(user_data);

    let create_info: rps::RpsDeviceCreateInfo = unsafe { std::mem::zeroed() };

    let mut rps_device: rps::RpsDevice = std::ptr::null_mut();

    map_result(unsafe { rps::rpsDeviceCreate(&create_info, &mut rps_device) }).unwrap();

    let runtime_create_info: rps::RpsRuntimeDeviceCreateInfo = unsafe { std::mem::zeroed() };

    map_result(unsafe {
        rps::addRuntime(
            &rps::RpsNullRuntimeDeviceCreateInfo {
                pDeviceCreateInfo: &create_info,
                pRuntimeCreateInfo: &runtime_create_info,
            },
            &mut rps_device,
            Some(create_command_resources),
            user_data_raw as *mut std::ffi::c_void,
        )
    })
    .unwrap();

    let mut graph_create_info: rps::RpsRenderGraphCreateInfo = unsafe { std::mem::zeroed() };

    let queue_flags = [
        rps::RpsQueueFlagBits_RPS_QUEUE_FLAG_GRAPHICS,
        rps::RpsQueueFlagBits_RPS_QUEUE_FLAG_COMPUTE,
        rps::RpsQueueFlagBits_RPS_QUEUE_FLAG_COPY,
    ];

    /*
    let lib = unsafe {
        libloading::Library::new("/home/ashley/projects/rust-rps/libout.so").unwrap()
    };

    let symbol: libloading::Symbol<DynamicLibraryInitFunction> = unsafe {
        lib.get(b"___rps_dyn_lib_init").unwrap()
    };

    map_result(unsafe {
        rps::rpsRpslDynamicLibraryInit(Some(*symbol.into_raw()))
    }).unwrap();

    let entry_name = b"rpsl_M_hello_triangle_E_main";

    let entry: libloading::Symbol<rps::RpsRpslEntry> = unsafe {
        lib.get(entry_name)
    }.unwrap();
    */

    let mut graph: rps::RpsRenderGraph = std::ptr::null_mut();

    graph_create_info.scheduleInfo.pQueueInfos = queue_flags.as_ptr();
    graph_create_info.scheduleInfo.numQueues = queue_flags.len() as u32;
    graph_create_info.mainEntryCreateInfo.hRpslEntryPoint = unsafe { rpsl_M_hello_triangle_E_main };

    let x = unsafe { *(rpsl_M_hello_triangle_E_main as *const RpslEntry) };

    map_result(unsafe { rps::rpsRenderGraphCreate(rps_device, &graph_create_info, &mut graph) })
        .unwrap();

    let subprogram = unsafe { rps::rpsRenderGraphGetMainEntry(graph) };

    let callback = rps::RpsCmdCallback {
        pfnCallback: Some(draw_triangle_cb),
        pUserContext: std::ptr::null_mut(),
        flags: 0,
    };

    map_result(unsafe {
        rps::rpsProgramBindNodeCallback(subprogram, b"Triangle\0".as_ptr() as *const i8, &callback)
    })
    .unwrap();


    let mut completed_frame_index = u64::max_value();
    let mut frame_index = 0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                let mut update_info: rps::RpsRenderGraphUpdateInfo = unsafe { std::mem::zeroed() };

                let back_buffer = rps::RpsResourceDesc {
                    type_: rps::RpsResourceType_RPS_RESOURCE_TYPE_IMAGE_2D,
                    temporalLayers: 1,
                    flags: 0,
                    __bindgen_anon_1: rps::RpsResourceDesc__bindgen_ty_1 {
                        image: rps::RpsResourceDesc__bindgen_ty_1__bindgen_ty_1 {
                            width: config.width,
                            height: config.height,
                            mipLevels: 1,
                            sampleCount: 1,
                            format: rps::RpsFormat_RPS_FORMAT_R8G8B8A8_UNORM,
                            __bindgen_anon_1: rps::RpsResourceDesc__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1 {
                                arrayLayers: 1,
                            },
                        },
                    },
                };

                let args: [rps::RpsConstant; 1] = [(&back_buffer) as *const rps::RpsResourceDesc as _];

                update_info.frameIndex = frame_index;
                update_info.gpuCompletedFrameIndex = completed_frame_index;
                update_info.numArgs = 1;
                update_info.ppArgs = args.as_ptr();

                map_result(unsafe { rps::rpsRenderGraphUpdate(graph, &update_info) }).unwrap();

                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let user_data = unsafe {
                        &*user_data_raw
                    };

                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: user_data.clear_colour[0] as f64,
                                    g: user_data.clear_colour[1] as f64,
                                    b: user_data.clear_colour[2] as f64,
                                    a: user_data.clear_colour[3] as f64,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

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
                        let record_info = rps::RpsRenderGraphRecordCommandInfo {
                            cmdBeginIndex: batch.cmdBegin,
                            numCmds: batch.numCmds,
                            frameIndex: frame_index,
                            flags: 0,
                            hCmdBuffer: rps::RpsRuntimeCommandBuffer_T {
                                ptr: (&mut rpass) as *mut wgpu::RenderPass as *mut std::ffi::c_void,
                            },
                            pUserContext: user_data_raw as *mut std::ffi::c_void,
                        };

                        map_result(unsafe {
                            rps::rpsRenderGraphRecordCommands(graph, &record_info)
                        })
                        .unwrap();
                    }

                    completed_frame_index = frame_index;
                    frame_index += 1;
                }

                queue.submit(Some(encoder.finish()));
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

fn map_result(rps_result: rps::RpsResult) -> Result<(), rps::RpsResult> {
    match rps_result {
        rps::RpsResult::RPS_OK => Ok(()),
        _ => Err(rps_result),
    }
}
