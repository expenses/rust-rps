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

fn main() {
    let opts = Opts::from_args();

    let file_stem = opts.filename.file_stem().unwrap().to_str().unwrap();

    let lib = unsafe { libloading::Library::new(&opts.filename).unwrap() };

    let symbol: libloading::Symbol<DynamicLibraryInitFunction> =
        unsafe { lib.get(b"___rps_dyn_lib_init").unwrap() };

    map_result(unsafe { rps::rpsRpslDynamicLibraryInit(Some(*symbol.into_raw())) }).unwrap();

    let entry_name = format!("rpsl_M_{}_E_{}", file_stem, opts.entry_point);

    let entry: libloading::Symbol<rps::RpsRpslEntry> = unsafe { lib.get(entry_name.as_bytes()) }.unwrap();

    let entry = unsafe { entry.into_raw().into_raw() as *const *const c_void };

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
    graph_create_info.mainEntryCreateInfo.hRpslEntryPoint = unsafe { (*entry) as *mut _ };

    /*dbg!(unsafe {
        *(graph_create_info.mainEntryCreateInfo.hRpslEntryPoint as *const RpslEntry)
    });*/

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
