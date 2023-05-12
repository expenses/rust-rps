use crate::{BorrowedOrOwned, CommandBuffer, Resource, UserData};
use rps::sys;
use rps::CmdCallbackContext;

pub unsafe extern "C" fn draw_triangle(context: *const sys::RpsCmdCallbackContext) {
    let context = CmdCallbackContext::<CommandBuffer, UserData>::new(context);

    let image_view = *(context.args[0] as *const sys::RpsImageView);

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

pub unsafe extern "C" fn geometry_pass(context: *const sys::RpsCmdCallbackContext) {
    let context = CmdCallbackContext::<CommandBuffer, UserData>::new(context);

    let image_view = context.reinterpret_arg_as::<sys::RpsImageView>(0);
    let mut aspect_ratio = *context.reinterpret_arg_as::<f32>(1);
    let time = *context.reinterpret_arg_as::<f32>(2);
    let viewport = *context.reinterpret_arg_as::<sys::RpsViewport>(3);

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

pub unsafe extern "C" fn upscale(context: *const sys::RpsCmdCallbackContext) {
    let context = CmdCallbackContext::<CommandBuffer, UserData>::new(context);

    let dest = context.reinterpret_arg_as::<sys::RpsImageView>(0);
    let source = context.reinterpret_arg_as::<sys::RpsImageView>(1);

    let mut source_res = context.resources[source.base.resourceId as usize].hRuntimeResource;
    let mut dest_res = context.resources[dest.base.resourceId as usize].hRuntimeResource;

    let source_res = &*(source_res.ptr as *const Resource);

    let dest_res = &*(dest_res.ptr as *const Resource);

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
