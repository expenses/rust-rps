#![allow(nonstandard_style)]

use bitflags::bitflags;
use std::ffi::c_void;

mod builtin_callbacks;
mod node_callbacks;

pub mod rps {
    #![allow(rustdoc::broken_intra_doc_links)]
    #![allow(clippy::missing_safety_doc)]
    #![allow(clippy::useless_transmute)]
    #![allow(clippy::transmute_int_to_bool)]
    #![allow(clippy::too_many_arguments)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    pub use root::*;
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct ShaderStages: u32 {
        const VS = rps::root::RpsShaderStageBits::RPS_SHADER_STAGE_VS as u32;
        const PS = rps::root::RpsShaderStageBits::RPS_SHADER_STAGE_PS as u32;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct AccessFlagBits: i32 {
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

extern "C" {
    static rpsl_M_rps_multithreading_E_main: rps::RpsRpslEntry;

    static rpsl_M_upscale_E_hello_rpsl: rps::RpsRpslEntry;
}

fn vector_to_slice<T, A>(vector: &rps::rps::Vector<T, A>) -> &[T] {
    unsafe { std::slice::from_raw_parts(vector.m_pArray, vector.m_Count) }
}

fn array_ref_to_mut_slice<T>(array_ref: &mut rps::rps::ArrayRef<T, u64>) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(array_ref.m_pData, array_ref.m_Size as usize) }
}

#[derive(Debug)]
struct UserData {
    triangle_pipeline: wgpu::RenderPipeline,
    multithreaded_triangle_pipeline: wgpu::RenderPipeline,
    device: wgpu::Device,
}

struct CommandBuffer {
    encoder: Option<wgpu::CommandEncoder>,
}

fn main() {
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

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::VERTEX,
            range: 0..4,
        }],
    });

    let user_data = Box::new(UserData {
        triangle_pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &device
                    .create_shader_module(wgpu::include_spirv!("../shaders/triangle.vert.spv")),
                entry_point: "VSMain",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &device
                    .create_shader_module(wgpu::include_spirv!("../shaders/triangle.frag.spv")),
                entry_point: "PSMain",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        }),
        multithreaded_triangle_pipeline: device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &device.create_shader_module(wgpu::include_spirv!(
                        "../shaders/triangle_breathing.vert.spv"
                    )),
                    entry_point: "VSMain",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &device.create_shader_module(wgpu::include_spirv!(
                        "../shaders/triangle_breathing.frag.spv"
                    )),
                    entry_point: "PSMain",
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            },
        ),
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

    let mut multithreading = true;

    if multithreading {
        graph_create_info.mainEntryCreateInfo.hRpslEntryPoint =
            unsafe { rpsl_M_rps_multithreading_E_main };
    } else {
        graph_create_info.mainEntryCreateInfo.hRpslEntryPoint =
            unsafe { rpsl_M_upscale_E_hello_rpsl };
    }

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

fn map_result(rps_result: rps::RpsResult) -> Result<(), rps::RpsResult> {
    match rps_result {
        rps::RpsResult::RPS_OK => Ok(()),
        _ => Err(rps_result),
    }
}

fn map_wgpu_format_to_rps(format: wgpu::TextureFormat) -> rps::RpsFormat {
    match format {
        wgpu::TextureFormat::Bgra8UnormSrgb => rps::RpsFormat::RPS_FORMAT_B8G8R8A8_UNORM_SRGB,
        other => panic!("{:?}", other),
    }
}

fn map_rps_format_to_wgpu(format: rps::RpsFormat) -> wgpu::TextureFormat {
    match format {
        rps::RpsFormat::RPS_FORMAT_B8G8R8A8_UNORM_SRGB => wgpu::TextureFormat::Bgra8UnormSrgb,
        other => panic!("{:?}", other),
    }
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
