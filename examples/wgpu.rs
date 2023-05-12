use rps::{bind_node_callback, load_dynamic_rps_library, map_result, sys};

use std::ffi::c_void;
use std::path::PathBuf;
use structopt::StructOpt;

mod builtin_callbacks;
mod node_callbacks;
mod pipelines;
mod reflection;

use pipelines::RenderPipeline;

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
    let entry_name = format!("rpsl_M_{}_E_{}", file_stem, opts.entry_point);
    let entry = load_dynamic_rps_library(&lib, &entry_name).unwrap();

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

    let mut device = rps::Device::new().unwrap();
    device
        .add_callback_runtime(
            sys::Callbacks {
                create_command_resources: Some(builtin_callbacks::create_command_resources),
                clear_color: Some(builtin_callbacks::clear_color),
                create_resources: Some(builtin_callbacks::create_resources),
                destroy_runtime_resource_deferred: Some(
                    builtin_callbacks::destroy_runtime_resource_deferred,
                ),
            },
            user_data_raw,
        )
        .unwrap();

    let graph = device.create_render_graph(entry).unwrap();

    let subprogram = graph.get_main_entry();

    if let Err(err) = bind_node_callback(
        subprogram,
        "GeometryPass",
        Some(node_callbacks::geometry_pass),
    ) {
        eprintln!("Error binding GeometryPass node: {:?}", err);
    }
    if let Err(err) =
        bind_node_callback(subprogram, "Triangle", Some(node_callbacks::draw_triangle))
    {
        eprintln!("Error binding triangle node: {:?}", err);
    }

    if let Err(err) = bind_node_callback(subprogram, "Upscale", Some(node_callbacks::upscale)) {
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
                let mut update_info: sys::RpsRenderGraphUpdateInfo = unsafe { std::mem::zeroed() };

                let back_buffer = sys::RpsResourceDesc {
                    type_: sys::RpsResourceType::RPS_RESOURCE_TYPE_IMAGE_2D,
                    temporalLayers: 1,
                    flags: 0,
                    __bindgen_anon_1: sys::RpsResourceDesc__bindgen_ty_1 {
                        image: sys::RpsResourceDesc__bindgen_ty_1__bindgen_ty_1 {
                            width: config.width,
                            height: config.height,
                            mipLevels: 1,
                            sampleCount: 1,
                            format: map_wgpu_format_to_rps(swapchain_format),
                            __bindgen_anon_1:
                                sys::RpsResourceDesc__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1 {
                                    arrayLayers: 1,
                                },
                        },
                    },
                };

                let time = (std::time::Instant::now() - start).as_secs_f32();

                let args: &[sys::RpsConstant] = &[
                    (&back_buffer) as *const sys::RpsResourceDesc as _,
                    (&time) as *const f32 as _,
                ];

                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");

                let backbuffer_res = Box::new(Resource::SurfaceFrame(
                    frame.texture.create_view(&Default::default()),
                ));

                let res = sys::RpsRuntimeResource {
                    ptr: Box::into_raw(backbuffer_res) as *mut c_void,
                };

                let res_p = (&res) as *const sys::RpsRuntimeResource;

                let arg_resources: &[*const sys::RpsRuntimeResource] = &[res_p];

                update_info.frameIndex = frame_index;
                update_info.gpuCompletedFrameIndex = completed_frame_index;
                update_info.numArgs = args.len() as u32;
                update_info.ppArgs = args.as_ptr();
                update_info.ppArgResources = arg_resources.as_ptr();
                if first_time {
                    update_info.diagnosticFlags =
                        sys::RpsDiagnosticFlagBits::RPS_DIAGNOSTIC_ENABLE_ALL as u32;
                }

                first_time = false;

                graph.update(&update_info).unwrap();

                {
                    let layout = graph.get_batch_layout().unwrap();

                    for batch in layout.command_batches() {
                        let encoder = user_data.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor { label: None },
                        );

                        let mut cb = CommandBuffer {
                            encoder: Some(encoder),
                        };

                        graph
                            .record_commands(frame_index, *batch, user_data_raw, &mut cb)
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

pub fn map_wgpu_format_to_rps(format: wgpu::TextureFormat) -> sys::RpsFormat {
    match format {
        wgpu::TextureFormat::Bgra8UnormSrgb => sys::RpsFormat::RPS_FORMAT_B8G8R8A8_UNORM_SRGB,
        other => panic!("{:?}", other),
    }
}

pub fn map_rps_format_to_wgpu(format: sys::RpsFormat) -> wgpu::TextureFormat {
    match format {
        sys::RpsFormat::RPS_FORMAT_B8G8R8A8_UNORM_SRGB => wgpu::TextureFormat::Bgra8UnormSrgb,
        other => panic!("{:?}", other),
    }
}
