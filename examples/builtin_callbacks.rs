use crate::{map_rps_format_to_wgpu, BorrowedOrOwned, CommandBuffer, Resource, UserData};
use rps::{
    mapping::{array_ref_to_mut_slice, AccessFlagBits},
    sys, CmdCallbackContext,
};
use std::ffi::c_void;

pub unsafe extern "C" fn create_command_resources(
    _context: *const c_void,
    _user_data: *mut c_void,
) -> sys::RpsResult {
    sys::RpsResult::RPS_OK
}

pub unsafe extern "C" fn create_resources(
    _context: *const c_void,
    array: *mut c_void,
    user_data: *mut c_void,
) -> sys::RpsResult {
    let user_data = &mut *(user_data as *mut UserData);

    let arr = array as *mut sys::rps::ArrayRef<sys::rps::ResourceInstance, u64>;

    let resources = unsafe { array_ref_to_mut_slice(&mut *arr) };

    for mut resource in resources.iter_mut() {
        if resource.isExternal() {
            continue;
        }

        if !resource.isPendingCreate() {
            continue;
        }

        let access = resource.allAccesses._base;

        let access_flags = AccessFlagBits::from_bits_retain(access.accessFlags as i32);

        match resource.desc.type_() {
            sys::RpsResourceType::RPS_RESOURCE_TYPE_IMAGE_2D => {
                let mut usage = wgpu::TextureUsages::empty();

                if access_flags.intersects(
                    AccessFlagBits::RENDER_TARGET
                        | AccessFlagBits::DEPTH_WRITE
                        | AccessFlagBits::STENCIL_WRITE,
                ) {
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

    sys::RpsResult::RPS_OK
}

pub unsafe extern "C" fn destroy_runtime_resource_deferred(
    resource: *mut c_void,
    _user_data: *mut c_void,
) {
    let mut resource = &mut *(resource as *mut sys::rps::ResourceInstance);

    debug_assert!(!resource.isExternal());

    let _ = Box::from_raw(resource.hRuntimeResource.ptr as *mut Resource);

    resource.hRuntimeResource.ptr = std::ptr::null_mut();
}

pub unsafe extern "C" fn clear_color(context: *const sys::RpsCmdCallbackContext) {
    let context = CmdCallbackContext::<CommandBuffer, UserData>::new(context);

    //dbg!(image_view);

    let image_view = context.reinterpret_arg_as::<sys::RpsImageView>(0);

    let clear_value = context.reinterpret_arg_as::<sys::RpsClearValue>(1);
    let clear_value = clear_value.color.float32;

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

    context
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
