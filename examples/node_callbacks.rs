use crate::{BorrowedOrOwned, CommandBuffer, Resource, UserData};
use glam::Mat4;
use rps_custom_backend::{ffi, rps, CmdCallbackContext};

pub unsafe extern "C" fn draw_triangle(context: *const rps::CmdCallbackContext) {
    let context = CmdCallbackContext::<CommandBuffer, UserData>::new(context);

    let image_view = *(context.args[0] as *const ffi::RpsImageView);

    let resource = context.resources[image_view.base.resourceId as usize]
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

    let mut render_pass = context
        .command_buffer
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

    render_pass.set_pipeline(&context.user_data.triangle_pipeline.pipeline);
    render_pass.draw(0..3, 0..1);
}

pub unsafe extern "C" fn geometry_pass(context: *const rps::CmdCallbackContext) {
    let context = CmdCallbackContext::<CommandBuffer, UserData>::new(context);

    let image_view = context.reinterpret_arg_as::<ffi::RpsImageView>(0);
    let mut aspect_ratio = *context.reinterpret_arg_as::<f32>(1);
    let time = *context.reinterpret_arg_as::<f32>(2);
    let viewport = *context.reinterpret_arg_as::<ffi::RpsViewport>(3);

    aspect_ratio *= time.sin().abs();

    let resource = context.resources[image_view.base.resourceId as usize]
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

    let mut render_pass = context
        .command_buffer
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

    render_pass.set_viewport(
        viewport.x,
        viewport.y,
        viewport.width,
        viewport.height,
        viewport.minZ,
        viewport.maxZ,
    );

    render_pass.set_pipeline(&context.user_data.multithreaded_triangle_pipeline.pipeline);
    render_pass.set_push_constants(
        wgpu::ShaderStages::VERTEX,
        0,
        bytemuck::bytes_of(&aspect_ratio),
    );
    render_pass.draw(0..3, 0..1);
}

pub unsafe extern "C" fn upscale(context: *const rps::CmdCallbackContext) {
    let context = CmdCallbackContext::<CommandBuffer, UserData>::new(context);

    let dest = context.reinterpret_arg_as::<ffi::RpsImageView>(0);
    let source = context.reinterpret_arg_as::<ffi::RpsImageView>(1);

    let source_res = context.resources[source.base.resourceId as usize]
        .hRuntimeResource
        .ptr;
    let dest_res = context.resources[dest.base.resourceId as usize]
        .hRuntimeResource
        .ptr;

    let source_res = &*(source_res as *const Resource);

    let dest_res = &*(dest_res as *const Resource);

    let source_tex_view = match source_res {
        Resource::Texture(texture) => {
            let texture_view = texture.create_view(&Default::default());
            BorrowedOrOwned::Owned(texture_view)
        }
        Resource::SurfaceFrame(texture_view) => BorrowedOrOwned::Borrowed(texture_view),
    };

    let dest_tex_view = match dest_res {
        Resource::Texture(texture) => {
            let texture_view = texture.create_view(&Default::default());
            BorrowedOrOwned::Owned(texture_view)
        }
        Resource::SurfaceFrame(texture_view) => BorrowedOrOwned::Borrowed(texture_view),
    };

    let bind_group = context
        .user_data
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &context.user_data.blit_pipeline.bind_group_layouts[&0],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&context.user_data.sampler),
                },
            ],
        });

    let mut render_pass = context
        .command_buffer
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

    render_pass.set_pipeline(&context.user_data.blit_pipeline.pipeline);
    render_pass.set_bind_group(0, &bind_group, &[]);
    render_pass.draw(0..3, 0..1);
}

pub unsafe extern "C" fn draw(context: *const rps::CmdCallbackContext) {
    let context = CmdCallbackContext::<CommandBuffer, UserData>::new(context);

    let image_view = *(context.args[0] as *const ffi::RpsImageView);
    let depth_view = *(context.args[1] as *const ffi::RpsImageView);

    let image_res = &context.resources[image_view.base.resourceId as usize];

    let img_desc = image_res.desc.__bindgen_anon_1.image;

    let resource = context.resources[image_view.base.resourceId as usize]
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

    let resource = context.resources[depth_view.base.resourceId as usize]
        .hRuntimeResource
        .ptr;

    let resource = &*(resource as *const Resource);

    let depth_texture_view = match resource {
        Resource::Texture(texture) => {
            let texture_view = texture.create_view(&Default::default());
            BorrowedOrOwned::Owned(texture_view)
        }
        Resource::SurfaceFrame(texture_view) => BorrowedOrOwned::Borrowed(texture_view),
    };

    let (vertex, indices, num_indices, tex) = &context.user_data.gltf;

    let bind_group = context
        .user_data
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &context.user_data.pipeline_3d.bind_group_layouts[&0],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&context.user_data.sampler),
                },
            ],
        });

    let mut render_pass = context
        .command_buffer
        .encoder
        .as_mut()
        .unwrap()
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

    let camera_rig = &context.user_data.camera_rig;

    let view_matrix = Mat4::look_at_rh(
        camera_rig.final_transform.position,
        camera_rig.final_transform.position + camera_rig.final_transform.forward(),
        camera_rig.final_transform.up(),
    );

    let perspective_matrix = Mat4::perspective_infinite_rh(
        59.0_f32.to_radians(),
        img_desc.width as f32 / img_desc.height as f32,
        0.001,
    );

    render_pass.set_pipeline(&context.user_data.pipeline_3d.pipeline);
    render_pass.set_bind_group(0, &bind_group, &[]);
    render_pass.set_vertex_buffer(0, vertex.slice(..));
    render_pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
    render_pass.set_push_constants(
        wgpu::ShaderStages::VERTEX,
        0,
        bytemuck::bytes_of(&(perspective_matrix * view_matrix)),
    );
    render_pass.draw_indexed(0..*num_indices, 0, 0..1);
    //render_pass.draw(0..3, 0..1);
}
