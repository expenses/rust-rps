use crate::{
    mapping::{array_ref_to_mut_slice, map_rps_format_to_wgpu, vector_to_slice, AccessFlagBits},
    rps, BorrowedOrOwned, CommandBuffer, Resource, UserData,
};
use std::ffi::c_void;

pub unsafe extern "C" fn create_command_resources(
    _context: *const c_void,
    _user_data: *mut c_void,
) -> rps::RpsResult {
    rps::RpsResult::RPS_OK
}

pub unsafe extern "C" fn create_resources(
    _context: *const c_void,
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

pub unsafe extern "C" fn destroy_runtime_resource_deferred(
    resource: *mut c_void,
    _user_data: *mut c_void,
) {
    let mut resource = &mut *(resource as *mut rps::rps::ResourceInstance);

    debug_assert!(!resource.isExternal());

    let _ = Box::from_raw(resource.hRuntimeResource.ptr as *mut Resource);

    resource.hRuntimeResource.ptr = std::ptr::null_mut();
}

pub unsafe extern "C" fn clear_color(context: *const rps::RpsCmdCallbackContext) {
    let context = *(context as *const rps::rps::RuntimeCmdCallbackContext);

    let resource_cache = &(*context.pRenderGraph).m_resourceCache;
    let resources = vector_to_slice(resource_cache);

    let base_context = context._base;

    let command_buffer = unsafe { &mut *(base_context.hCommandBuffer.ptr as *mut CommandBuffer) };

    let args = std::slice::from_raw_parts(base_context.ppArgs, base_context.numArgs as usize);

    let image_view = *(args[0] as *const rps::RpsImageView);

    let clear_value = *(args[1] as *const rps::RpsClearValue);
    let clear_value = clear_value.color.float32;

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
