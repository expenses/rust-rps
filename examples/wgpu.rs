use glam::{Vec2, Vec3};
use winit::event::*;

use std::path::PathBuf;
use structopt::StructOpt;
use wgpu::util::DeviceExt;

mod builtin_callbacks;
mod node_callbacks;
mod pipelines;
mod reflection;

use pipelines::RenderPipeline;

use rps_custom_backend::{ffi, rps};

struct UserData {
    triangle_pipeline: RenderPipeline,
    multithreaded_triangle_pipeline: RenderPipeline,
    pipeline_3d: RenderPipeline,
    blit_pipeline: RenderPipeline,
    device: wgpu::Device,
    sampler: wgpu::Sampler,
    camera_rig: dolly::rig::CameraRig,
    gltf: (wgpu::Buffer, wgpu::Buffer, u32, wgpu::TextureView),
}

struct CommandBuffer {
    encoder: Option<wgpu::CommandEncoder>,
}

#[derive(StructOpt)]
struct Opts {
    filename: PathBuf,
    entry_point: String,
}

pub fn bind_node_callback(
    subprogram: rps::Subprogram,
    entry_point: &str,
    callback: rps::PfnCmdCallback,
) -> Result<(), rps::Result> {
    let entry_point = std::ffi::CString::new(entry_point).unwrap();

    unsafe {
        rps::program_bind_node_callback(
            subprogram,
            entry_point.as_ptr(),
            &rps::CmdCallback {
                pfn_callback: callback,
                ..Default::default()
            },
        )
    }
}

fn main() -> anyhow::Result<()> {
    unsafe {
        let opts = Opts::from_args();

        let file_stem = opts.filename.file_stem().unwrap().to_str().unwrap();

        let lib = unsafe { libloading::Library::new(&opts.filename).unwrap() };
        let entry_name = format!("rpsl_M_{}_E_{}", file_stem, opts.entry_point);
        let entry = rps::load_dynamic_library_and_get_entry_point(&lib, &entry_name).unwrap();

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
                    max_push_constant_size: 64,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
        ))
        .unwrap();

        let gltf = load_gltf_from_bytes(
            include_bytes!("../bloom/bloom_example.glb"),
            &device,
            &queue,
        )?;

        let mut keyboard_state = KeyboardState::default();
        let mut fullscreen = false;

        //let mut cursor_grab = false;

        let mut camera_rig: dolly::rig::CameraRig = dolly::rig::CameraRig::builder()
            .with(dolly::drivers::Position::new(dolly::glam::Vec3::new(
                2.0, 4.0, 1.0,
            )))
            .with(dolly::drivers::YawPitch::new().pitch_degrees(-74.0))
            .with(dolly::drivers::Smooth::new_position_rotation(0.5, 0.5))
            .build();

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

        let attrs = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

        let user_data = Box::new(UserData {
            multithreaded_triangle_pipeline: RenderPipeline::new(
                &device,
                "shaders/triangle_breathing.vert.spv",
                "shaders/triangle_breathing.frag.spv",
                &[],
                &[Some(swapchain_format.into())],
                None,
            ),
            triangle_pipeline: RenderPipeline::new(
                &device,
                "shaders/triangle.vert.spv",
                "shaders/triangle.frag.spv",
                &[],
                &[Some(swapchain_format.into())],
                None,
            ),
            blit_pipeline: RenderPipeline::new(
                &device,
                "shaders/blit.vert.spv",
                "shaders/blit.frag.spv",
                &[],
                &[Some(swapchain_format.into())],
                None,
            ),
            pipeline_3d: RenderPipeline::new(
                &device,
                "bloom/shaders/vertex.spv",
                "bloom/shaders/fragment.spv",
                &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as _,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
                &[Some(wgpu::TextureFormat::Rgba16Float.into())],
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
            ),
            sampler: device.create_sampler(&Default::default()),
            device,
            camera_rig,
            gltf,
        });

        let user_data_raw = Box::into_raw(user_data);

        let device_create_info = rps::DeviceCreateInfo::default();

        let device = unsafe { rps::device_create(&device_create_info) }.unwrap();

        rps_custom_backend::add_callback_runtime(
            &device,
            &device_create_info,
            ffi::Callbacks {
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

        let queues = &[rps::QueueFlags::all()];

        let mut x = rps::RenderGraphCreateInfo::default();

        x.schedule_info.queue_infos = queues.as_ptr();
        x.schedule_info.num_queues = queues.len() as u32;
        x.main_entry_create_info.rpsl_entry_point = entry;

        let graph = unsafe { rps::render_graph_create(device, &x) }.unwrap();

        let subprogram = rps::render_graph_get_main_entry(graph);

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

        if let Err(err) = bind_node_callback(
            subprogram,
            "tonemap_and_blit",
            Some(node_callbacks::upscale),
        ) {
            eprintln!("Error binding upscale node: {:?}", err);
        };

        if let Err(err) = bind_node_callback(subprogram, "draw", Some(node_callbacks::draw)) {
            eprintln!("Error binding draw node: {:?}", err);
        };

        let mut completed_frame_index = u64::max_value();
        let mut frame_index = 0;
        let mut first_time = true;

        event_loop.run(move |event, _, control_flow| {
            let user_data = unsafe { &mut *user_data_raw };

            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        let pressed = input.state == ElementState::Pressed;

                        match input.virtual_keycode {
                            Some(VirtualKeyCode::W | VirtualKeyCode::Up) => {
                                keyboard_state.forwards = pressed;
                            }
                            Some(VirtualKeyCode::A | VirtualKeyCode::Left) => {
                                keyboard_state.left = pressed;
                            }
                            Some(VirtualKeyCode::S | VirtualKeyCode::Down) => {
                                keyboard_state.backwards = pressed;
                            }
                            Some(VirtualKeyCode::D | VirtualKeyCode::Right) => {
                                keyboard_state.right = pressed;
                            }
                            Some(VirtualKeyCode::G) => {
                                if pressed {
                                    keyboard_state.cursor_grab = !keyboard_state.cursor_grab;

                                    if keyboard_state.cursor_grab {
                                        // Try both methods of grabbing the cursor.
                                        let result = window
                                            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                                            .or_else(|_| {
                                                window.set_cursor_grab(
                                                    winit::window::CursorGrabMode::Confined,
                                                )
                                            });

                                        if let Err(error) = result {
                                            eprintln!(
                                            "Got an error when trying to set the cursor grab: {}",
                                            error
                                        );
                                        }
                                    } else {
                                        // This can't fail.
                                        let _ = window
                                            .set_cursor_grab(winit::window::CursorGrabMode::None);
                                    }
                                    window.set_cursor_visible(!keyboard_state.cursor_grab);
                                }
                            }
                            Some(VirtualKeyCode::LControl | VirtualKeyCode::RControl) => {
                                keyboard_state.control = pressed
                            }
                            Some(VirtualKeyCode::F) => {
                                if pressed && keyboard_state.control {
                                    fullscreen = !fullscreen;

                                    window.set_fullscreen(if fullscreen {
                                        Some(winit::window::Fullscreen::Borderless(Some(
                                            window.current_monitor().unwrap(),
                                        )))
                                    } else {
                                        None
                                    })
                                }
                            }
                            _ => {}
                        }
                    }
                    winit::event::WindowEvent::Resized(size) => {
                        // Reconfigure the surface with the new size
                        config.width = size.width;
                        config.height = size.height;
                        surface.configure(&user_data.device, &config);
                        // On macos the window needs to be redrawn manually after resizing
                        window.request_redraw();
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    _ => {}
                },
                winit::event::Event::DeviceEvent { event, .. } => match event {
                    winit::event::DeviceEvent::MouseMotion {
                        delta: (delta_x, delta_y),
                    } => {
                        if keyboard_state.cursor_grab {
                            user_data
                                .camera_rig
                                .driver_mut::<dolly::drivers::YawPitch>()
                                .rotate_yaw_pitch(-0.1 * delta_x as f32, -0.1 * delta_y as f32);
                        }
                    }
                    _ => {}
                },
                winit::event::Event::MainEventsCleared => {
                    {
                        let forwards =
                            keyboard_state.forwards as i32 - keyboard_state.backwards as i32;
                        let right = keyboard_state.right as i32 - keyboard_state.left as i32;

                        let move_vec = user_data.camera_rig.final_transform.rotation
                            * Vec3::new(right as f32, 0.0, -forwards as f32).clamp_length_max(1.0);

                        let delta_time = 1.0 / 60.0;
                        let speed = 3.0;

                        user_data
                            .camera_rig
                            .driver_mut::<dolly::drivers::Position>()
                            .translate(move_vec * delta_time * speed);

                        user_data.camera_rig.update(delta_time);
                    }

                    window.request_redraw();
                }
                winit::event::Event::RedrawRequested(_) => {
                    let time = (std::time::Instant::now() - start).as_secs_f32();

                    let back_buffer = rps::ResourceDesc {
                        ty: rps::ResourceType::IMAGE_2D,
                        temporal_layers: 1,
                        flags: Default::default(),
                        buffer_image: rps::ResourceBufferImageDesc {
                            image: rps::ResourceImageDesc {
                                width: config.width,
                                height: config.height,
                                mip_levels: 1,
                                sample_count: 1,
                                format: map_wgpu_format_to_rps(swapchain_format),
                                depth_or_array_layers: 1,
                            },
                        },
                    };

                    let args: &[rps::Constant] = &[
                        (&back_buffer) as *const rps::ResourceDesc as _,
                        (&time) as *const f32 as _,
                    ];

                    let frame = surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");

                    let backbuffer_ptr = Box::into_raw(Box::new(Resource::SurfaceFrame(
                        frame.texture.create_view(&Default::default()),
                    )));

                    let arg_resources = &[(&backbuffer_ptr) as *const *mut Resource as _];

                    let update_info = rps::RenderGraphUpdateInfo {
                        frame_index,
                        gpu_completed_frame_index: completed_frame_index,
                        diagnostic_flags: if first_time {
                            rps::DiagnosticFlags::all()
                        } else {
                            rps::DiagnosticFlags::empty()
                        },
                        num_args: args.len() as u32,
                        args: args.as_ptr(),
                        arg_resources: arg_resources.as_ptr(),
                        ..Default::default()
                    };

                    first_time = false;

                    rps::render_graph_update(graph, &update_info).unwrap();

                    let layout = rps::render_graph_get_batch_layout(graph).unwrap();

                    for batch in layout.cmd_batches() {
                        let encoder = user_data.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor { label: None },
                        );

                        let mut cb = CommandBuffer {
                            encoder: Some(encoder),
                        };

                        let cb_ptr = &cb as *const CommandBuffer;

                        rps::render_graph_record_commands(
                            graph,
                            &rps::RenderGraphRecordCommandInfo {
                                user_context: user_data_raw as *mut std::ffi::c_void,
                                cmd_buffer: rps::RuntimeCommandBuffer::from_raw(cb_ptr as _),
                                frame_index,
                                cmd_begin_index: batch.cmd_begin,
                                num_cmds: batch.num_cmds,
                                flags: Default::default(),
                            },
                        )
                        .unwrap();

                        let encoder = cb.encoder.take().unwrap();

                        queue.submit(Some(encoder.finish()));
                    }

                    completed_frame_index = frame_index;
                    frame_index += 1;

                    frame.present();
                }
                _ => {}
            }
        });
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

pub fn map_wgpu_format_to_rps(format: wgpu::TextureFormat) -> rps::Format {
    match format {
        wgpu::TextureFormat::Bgra8UnormSrgb => rps::Format::B8G8R8A8_UNORM_SRGB,
        other => panic!("{:?}", other),
    }
}

pub fn map_rps_format_to_wgpu(format: rps::Format) -> wgpu::TextureFormat {
    match format {
        rps::Format::B8G8R8A8_UNORM_SRGB => wgpu::TextureFormat::Bgra8UnormSrgb,
        rps::Format::R16G16B16A16_FLOAT => wgpu::TextureFormat::Rgba16Float,
        rps::Format::D32_FLOAT => wgpu::TextureFormat::Depth32Float,
        other => panic!("{:?}", other),
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec3,
    uv: Vec2,
}

fn load_gltf_from_bytes(
    bytes: &[u8],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<(wgpu::Buffer, wgpu::Buffer, u32, wgpu::TextureView)> {
    let gltf = gltf::Gltf::from_slice(bytes)?;

    let buffer_blob = gltf.blob.as_ref().unwrap();

    let mut indices = Vec::new();
    let mut vertices = Vec::new();

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|_| Some(buffer_blob));

            let read_indices = reader.read_indices().unwrap().into_u32();

            let num_vertices = vertices.len() as u32;

            indices.extend(read_indices.map(|index| index + num_vertices));

            let positions = reader.read_positions().unwrap();
            let uvs = reader.read_tex_coords(0).unwrap().into_f32();

            for (position, uv) in positions.zip(uvs) {
                vertices.push(Vertex {
                    position: position.into(),
                    uv: uv.into(),
                });
            }
        }
    }

    let material = gltf.materials().next().unwrap();

    let texture = material.emissive_texture().unwrap();

    let texture = load_texture_from_gltf(
        texture.texture(),
        "emissive texture",
        buffer_blob,
        device,
        queue,
    )?;

    let num_indices = indices.len() as u32;

    let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertices"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("indices"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    Ok((vertices, indices, num_indices, texture))
}

fn load_texture_from_gltf(
    texture: gltf::texture::Texture,
    label: &str,
    buffer_blob: &[u8],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<wgpu::TextureView> {
    let texture_view = match texture.source().source() {
        gltf::image::Source::View { view, .. } => view,
        _ => {
            return Err(anyhow::anyhow!(
                "Image source is a uri which we don't support"
            ))
        }
    };

    let texture_start = texture_view.offset();
    let texture_end = texture_start + texture_view.length();
    let texture_bytes = &buffer_blob[texture_start..texture_end];

    let decoded_bytes =
        image::load_from_memory_with_format(texture_bytes, image::ImageFormat::Png)?;

    let decoded_rgba8 = decoded_bytes.to_rgba8();

    Ok(device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: decoded_rgba8.width(),
                    height: decoded_rgba8.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            &*decoded_rgba8,
        )
        .create_view(&Default::default()))
}

#[derive(Default)]
struct KeyboardState {
    forwards: bool,
    right: bool,
    left: bool,
    backwards: bool,
    cursor_grab: bool,
    control: bool,
}
