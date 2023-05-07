use crate::{mapping::vector_to_slice, rps, BorrowedOrOwned, CommandBuffer, Resource, UserData};

pub unsafe extern "C" fn draw_triangle(context: *const rps::RpsCmdCallbackContext) {
    let context = *(context as *const rps::rps::RuntimeCmdCallbackContext);
    let base_context = context._base;

    let args =
        unsafe { std::slice::from_raw_parts(base_context.ppArgs, base_context.numArgs as usize) };

    let image_view = *(args[0] as *const rps::RpsImageView);

    let user_data = unsafe { &*(base_context.pUserRecordContext as *mut UserData) };

    let command_buffer = unsafe { &mut *(base_context.hCommandBuffer.ptr as *mut CommandBuffer) };

    let resource_cache = &(*context.pRenderGraph).m_resourceCache;
    let resources = vector_to_slice(resource_cache);

    let resource = resources[image_view.base.resourceId as usize]
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

    render_pass.set_pipeline(&user_data.triangle_pipeline.pipeline);
    render_pass.draw(0..3, 0..1);
}

pub unsafe extern "C" fn geometry_pass(context: *const rps::RpsCmdCallbackContext) {
    let context = *(context as *const rps::rps::RuntimeCmdCallbackContext);
    let base_context = context._base;

    let args =
        unsafe { std::slice::from_raw_parts(base_context.ppArgs, base_context.numArgs as usize) };

    let image_view = *(args[0] as *const rps::RpsImageView);
    let mut aspect_ratio = unsafe { *(args[1] as *const f32) };
    let time = unsafe { *(args[2] as *const f32) };

    let viewport = unsafe { *(args[3] as *const rps::RpsViewport) };

    aspect_ratio *= time.sin().abs();

    let user_data = unsafe { &*(base_context.pUserRecordContext as *mut UserData) };

    let command_buffer = unsafe { &mut *(base_context.hCommandBuffer.ptr as *mut CommandBuffer) };

    let resource_cache = &(*context.pRenderGraph).m_resourceCache;
    let resources = vector_to_slice(resource_cache);

    let resource = resources[image_view.base.resourceId as usize]
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

    render_pass.set_viewport(
        viewport.x,
        viewport.y,
        viewport.width,
        viewport.height,
        viewport.minZ,
        viewport.maxZ,
    );

    render_pass.set_pipeline(&user_data.multithreaded_triangle_pipeline.pipeline);
    render_pass.set_push_constants(
        wgpu::ShaderStages::VERTEX,
        0,
        bytemuck::bytes_of(&aspect_ratio),
    );
    render_pass.draw(0..3, 0..1);
}

pub unsafe extern "C" fn upscale(context: *const rps::RpsCmdCallbackContext) {
    let context = *(context as *const rps::rps::RuntimeCmdCallbackContext);
    let base_context = context._base;
    let args = std::slice::from_raw_parts(base_context.ppArgs, base_context.numArgs as usize);
    let resource_cache = &(*context.pRenderGraph).m_resourceCache;
    let resources = vector_to_slice(resource_cache);
    let command_buffer = unsafe { &mut *(base_context.hCommandBuffer.ptr as *mut CommandBuffer) };

    let user_data = unsafe { &*(base_context.pUserRecordContext as *mut UserData) };

    let dest = *(args[0] as *const rps::RpsImageView);
    let source = *(args[1] as *const rps::RpsImageView);

    let mut source_res = resources[source.base.resourceId as usize].hRuntimeResource;
    let mut dest_res = resources[dest.base.resourceId as usize].hRuntimeResource;

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

    let bind_group = user_data
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &user_data.blit_pipeline.bind_group_layouts[&0],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&user_data.sampler),
                },
            ],
        });

    let mut render_pass =
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

    render_pass.set_pipeline(&user_data.blit_pipeline.pipeline);
    render_pass.set_bind_group(0, &bind_group, &[]);
    render_pass.draw(0..3, 0..1);
}
