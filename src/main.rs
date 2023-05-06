#![allow(nonstandard_style)]

use bitflags::bitflags;
use std::borrow::Cow;
use std::ffi::c_void;

pub mod rps {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(rustdoc::broken_intra_doc_links)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    pub use root::*;
}

use std::mem::ManuallyDrop;

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
    static rpsl_M_rps_multithreading_E_main: rps::RpsRpslEntry;

    //#[no_mangle]
    //static rpsl_M_hello_triangle_E_mainBreathing: rps::RpsRpslEntry;

    static rpsl_M_upscale_E_hello_rpsl: rps::RpsRpslEntry;
}

fn vector_to_slice<T, A>(vector: &rps::rps::Vector<T, A>) -> &[T] {
    unsafe { std::slice::from_raw_parts(vector.m_pArray, vector.m_Count) }
}

fn array_ref_to_slice<T>(array_ref: &rps::rps::ArrayRef<T, u64>) -> &[T] {
    unsafe { std::slice::from_raw_parts(array_ref.m_pData, array_ref.m_Size as usize) }
}

fn array_ref_to_mut_slice<T>(array_ref: &mut rps::rps::ArrayRef<T, u64>) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(array_ref.m_pData, array_ref.m_Size as usize) }
}

unsafe extern "C" fn destroy_runtime_resource_deferred(
    resource: *mut c_void,
    user_data: *mut c_void,
) {
    let mut resource = &mut *(resource as *mut rps::rps::ResourceInstance);

    debug_assert!(!resource.isExternal());

    let boxed = Box::from_raw(resource.hRuntimeResource.ptr as *mut Resource);

    resource.hRuntimeResource.ptr = std::ptr::null_mut();
}

unsafe extern "C" fn upscale_cb(context: *const rps::RpsCmdCallbackContext) {
    let context = *(context as *const rps::rps::RuntimeCmdCallbackContext);
    let base_context = context._base;
    let args = std::slice::from_raw_parts(base_context.ppArgs, base_context.numArgs as usize);
    let resource_cache = &(*context.pRenderGraph).m_resourceCache;
    let resources = vector_to_slice(resource_cache);
    let command_buffer = unsafe { &mut *(base_context.hCommandBuffer.ptr as *mut CommandBuffer) };

    let dest = *(args[0] as *const rps::RpsImageView);
    let source = *(args[1] as *const rps::RpsImageView);

    let mut source_res = resources[source.base.resourceId as usize].hRuntimeResource;
    let mut dest_res = resources[dest.base.resourceId as usize].hRuntimeResource;

    let dest_res = &*(dest_res.ptr as *const Resource);

    let dest_tex_view = match dest_res {
        Resource::Texture(texture) => {
            let texture_view = texture.create_view(&Default::default());
            BorrowedOrOwned::Owned(texture_view)
        }
        Resource::SurfaceFrame(texture_view) => BorrowedOrOwned::Borrowed(texture_view),
    };

    command_buffer
        .encoder
        .as_mut()
        .unwrap()
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dest_tex_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
}

unsafe extern "C" fn clear_color(context: *const rps::RpsCmdCallbackContext) {
    let context = *(context as *const rps::rps::RuntimeCmdCallbackContext);

    let resource_cache = &(*context.pRenderGraph).m_resourceCache;
    let resources = vector_to_slice(resource_cache);

    let base_context = context._base;

    let user_data = unsafe { &*(base_context.pUserRecordContext as *mut UserData) };

    let command_buffer = unsafe { &mut *(base_context.hCommandBuffer.ptr as *mut CommandBuffer) };

    let args = std::slice::from_raw_parts(base_context.ppArgs, base_context.numArgs as usize);

    let image_view = *(args[0] as *const rps::RpsImageView);

    let clear_value = *(args[1] as *const rps::RpsClearValue);
    let clear_value = clear_value.color.float32;

    let mut resource = resources[image_view.base.resourceId as usize]
        .hRuntimeResource
        .ptr;
    let resource = &*(resource as *const Resource);

    let texture_view = match resource {
        Resource::Texture(texture) => {
            let texture_view = texture.create_view(&Default::default());
            BorrowedOrOwned::Owned(texture_view)
        }
        Resource::SurfaceFrame(texture_view) => BorrowedOrOwned::Borrowed(texture_view),
    };

    command_buffer
        .encoder
        .as_mut()
        .unwrap()
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_value[0] as f64,
                        g: clear_value[1] as f64,
                        b: clear_value[2] as f64,
                        a: clear_value[3] as f64,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
}

unsafe extern "C" fn draw_triangle_cb(context: *const rps::RpsCmdCallbackContext) {
    let context = *(context as *const rps::rps::RuntimeCmdCallbackContext);
    let base_context = context._base;

    let args =
        unsafe { std::slice::from_raw_parts(base_context.ppArgs, base_context.numArgs as usize) };

    let image_view = *(args[0] as *const rps::RpsImageView);

    let user_data = unsafe { &*(base_context.pUserRecordContext as *mut UserData) };

    let command_buffer = unsafe { &mut *(base_context.hCommandBuffer.ptr as *mut CommandBuffer) };

    let resource_cache = &(*context.pRenderGraph).m_resourceCache;
    let resources = vector_to_slice(resource_cache);

    let mut resource = resources[image_view.base.resourceId as usize]
        .hRuntimeResource
        .ptr;
    let resource = &*(resource as *const Resource);

    let texture_view = match resource {
        Resource::Texture(texture) => {
            let texture_view = texture.create_view(&Default::default());
            BorrowedOrOwned::Owned(texture_view)
        }
        Resource::SurfaceFrame(texture_view) => BorrowedOrOwned::Borrowed(texture_view),
    };

    let mut render_pass =
        command_buffer
            .encoder
            .as_mut()
            .unwrap()
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

    render_pass.set_pipeline(&user_data.triangle_pipeline);
    render_pass.draw(0..3, 0..1);
}

unsafe extern "C" fn create_resources(
    context: *const c_void,
    array: *mut c_void,
    user_data: *mut c_void,
) -> rps::RpsResult {
    let user_data = &mut *(user_data as *mut UserData);

    let arr = array as *mut rps::rps::ArrayRef<rps::rps::ResourceInstance, u64>;

    let resources = unsafe { array_ref_to_mut_slice(&mut *arr) };

    for mut resource in resources.iter_mut() {
        if resource.isExternal() {
            continue;
        }

        let access = resource.allAccesses._base;

        let access_flags = AccessFlagBits::from_bits_retain(access.accessFlags as i32);

        match resource.desc.type_() {
            rps::root::RpsResourceType::RPS_RESOURCE_TYPE_IMAGE_2D => {
                let mut usage = wgpu::TextureUsages::empty();

                if access_flags.contains(AccessFlagBits::RENDER_TARGET) {
                    usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
                }

                if access_flags.contains(AccessFlagBits::SHADER_RESOURCE) {
                    usage |= wgpu::TextureUsages::TEXTURE_BINDING;
                }

                if usage.is_empty() {
                    //eprintln!("Texture has no usages");
                    usage |= wgpu::TextureUsages::TEXTURE_BINDING;
                }

                let image = resource.desc.__bindgen_anon_1.image;

                let texture = user_data.device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d {
                        width: image.width,
                        height: image.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: image.mipLevels(),
                    sample_count: image.sampleCount(),
                    dimension: wgpu::TextureDimension::D2,
                    format: map_rps_format_to_wgpu(image.format()),
                    view_formats: &[map_rps_format_to_wgpu(image.format())],
                    usage,
                });

                resource.allocPlacement.heapId = 0;
                resource.hRuntimeResource.ptr = Box::into_raw(Box::new(texture)) as _;
                resource.prevFinalAccess = resource.initialAccess;
                resource.set_isPendingCreate(false);
            }
            _ => panic!(),
        }
    }

    rps::RpsResult::RPS_OK
}

unsafe extern "C" fn create_command_resources(
    context: *const c_void,
    user_data: *mut c_void,
) -> rps::RpsResult {
    rps::RpsResult::RPS_OK
} /*
  extern "C" fn mt_cb(context: *const rps::RpsCmdCallbackContext) {
      let context = unsafe { *context };

      let args = unsafe { std::slice::from_raw_parts(context.ppArgs, context.numArgs as usize) };

      let mut aspect_ratio = unsafe { *(args[1] as *const f32) };
      let time = unsafe { *(args[2] as *const f32) };

      let viewport = unsafe { *(args[3] as *const rps::RpsViewport) };

      aspect_ratio *= time.sin().abs();

      let user_data = unsafe { &*(context.pUserRecordContext as *mut UserData) };

      let command_buffer = unsafe { &mut *(context.hCommandBuffer.ptr as *mut CommandBuffer) };

      let mut render_pass =
          command_buffer
              .encoder
              .as_mut()
              .unwrap()
              .begin_render_pass(&wgpu::RenderPassDescriptor {
                  label: None,
                  color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                      view: user_data.backbuffer_view.as_ref().unwrap(),
                      resolve_target: None,
                      ops: wgpu::Operations {
                          load: wgpu::LoadOp::Load,
                          store: true,
                      },
                  })],
                  depth_stencil_attachment: None,
              });

      render_pass.set_viewport(
          viewport.x,
          viewport.y,
          viewport.width,
          viewport.height,
          viewport.minZ,
          viewport.maxZ,
      );

      render_pass.set_pipeline(&user_data.triangle_pipeline);
      render_pass.set_push_constants(
          wgpu::ShaderStages::VERTEX,
          0,
          bytemuck::bytes_of(&aspect_ratio),
      );
      render_pass.draw(0..3, 0..1);
  }*/

#[derive(Debug)]
struct UserData {
    triangle_pipeline: wgpu::RenderPipeline,
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

    let vert_shader = device.create_shader_module(wgpu::include_spirv!("../triangle.vert.spv"));
    let frag_shader = device.create_shader_module(wgpu::include_spirv!("../triangle.frag.spv"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::VERTEX,
            range: 0..4,
        }],
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

    let user_data = Box::new(UserData {
        triangle_pipeline: render_pipeline,
        device,
    });

    let user_data_raw = Box::into_raw(user_data);

    let create_info: rps::RpsDeviceCreateInfo = unsafe { std::mem::zeroed() };

    let mut rps_device: rps::RpsDevice = std::ptr::null_mut();

    map_result(unsafe { rps::rpsDeviceCreate(&create_info, &mut rps_device) }).unwrap();

    let runtime_create_info: rps::RpsRuntimeDeviceCreateInfo = unsafe { std::mem::zeroed() };

    map_result(unsafe {
        rps::add_callback_runtime(
            &create_info,
            &mut rps_device,
            rps::Callbacks {
                create_command_resources: Some(create_command_resources),
                clear_color: Some(clear_color),
                create_resources: Some(create_resources),
                destroy_runtime_resource_deferred: Some(destroy_runtime_resource_deferred),
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
    graph_create_info.mainEntryCreateInfo.hRpslEntryPoint = unsafe { rpsl_M_upscale_E_hello_rpsl };

    map_result(unsafe { rps::rpsRenderGraphCreate(rps_device, &graph_create_info, &mut graph) })
        .unwrap();

    let subprogram = unsafe { rps::rpsRenderGraphGetMainEntry(graph) };

    /*map_result(unsafe {
        rps::rpsProgramBindNodeCallback(
            subprogram,
            b"GeometryPass\0".as_ptr() as _,
            &rps::RpsCmdCallback {
                pfnCallback: Some(mt_cb),
                pUserContext: std::ptr::null_mut(),
                flags: 0,
            },
        )
    })
    .unwrap();*/
    map_result(unsafe {
        rps::rpsProgramBindNodeCallback(
            subprogram,
            b"Triangle\0".as_ptr() as _,
            &rps::RpsCmdCallback {
                pfnCallback: Some(draw_triangle_cb),
                pUserContext: std::ptr::null_mut(),
                flags: 0,
            },
        )
    })
    .unwrap();

    if let Err(err) = map_result(unsafe {
        rps::rpsProgramBindNodeCallback(
            subprogram,
            b"Upscale\0".as_ptr() as _,
            &rps::RpsCmdCallback {
                pfnCallback: Some(upscale_cb),
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
                    //(&time) as *const f32 as _,
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
                update_info.numArgs = args.len() as u32 + 1;
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
            Self::Owned(owned) => &owned,
            Self::Borrowed(borrowed) => borrowed,
        }
    }
}
