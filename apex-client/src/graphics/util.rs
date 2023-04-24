use std::process::ExitCode;

use log::info;
use wcore::graphics::context::{Graphics, Layout};
use winit::window::Window;

use crate::config::Config;

pub async fn new_graphics(window: &Window, config: &Config) -> Graphics {
    let backend = config.backend.into();
    let instance = wgpu::Instance::new(
        wgpu::InstanceDescriptor {
            backends             : backend,
            dx12_shader_compiler : Default::default(),
        }
    );
    
    #[cfg(not(target_arch = "wasm32"))]
    if config.gpus {
        println!("Available GPUs:");
        for (i, adapter) in instance.enumerate_adapters(backend).enumerate() {
            let info = adapter.get_info();
            println!("- [{i}] {}", info.name);
        }

        if !config.modes { // Allow combining both flags :')
            ExitCode::SUCCESS.exit_process();
        }
    }

    // # Safety
    // The surface needs to live as long as the window that created it.
    // State owns the window so this should be safe.
    let surface = unsafe { instance.create_surface(&window) }
        .expect("Failed to create a surface");

    let request_adapter = || async {
        instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference       : config.power_preference.into(),
                compatible_surface     : Some(&surface),
                force_fallback_adapter : false,
        }).await.expect("Failed to retrive an adapter")
    };

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let adapter = request_adapter().await;
        } else {
            let adapter = if let Some(gpu) = config.gpu {
                let error = format!("Failed to find graphics adapter [{}]", gpu);
                instance.enumerate_adapters(backend).nth(gpu).expect(&error)
            } else { request_adapter().await };
        }
    }

    let info = adapter.get_info();
    info!("Selected GPU: {} | ({:?})", info.name, info.device_type);
    info!("Selected backend: {:?}", info.backend);
    info!("Driver: {} | {}", info.driver, info.driver_info);

    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label:    None,
            features: wgpu::Features::empty(),

            // WebGL doesn't support all of wgpu's features, so if
            // we're building for the web we'll have to disable some.
            limits: {
                let mut limits = if cfg!(target_arch = "wasm32") {
                    wgpu::Limits {
                        // firefox gets angry otherwise...
                        max_vertex_attributes: 8,
                        .. wgpu::Limits::downlevel_webgl2_defaults()
                    }
                } else { wgpu::Limits::default() };

                limits.max_bind_groups = 8;

                limits
            },
        },
        None, // Trace path
    ).await.expect("Failed to retrive a device");

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter().copied()
        .find(|f| *f == wgpu::TextureFormat::Rgba8UnormSrgb)
        .unwrap_or(surface_caps.formats[0]);

    info!("Surface format: {:?}", surface_format);

    let size = window.inner_size();
    assert_ne!(size.width, 0);
    assert_ne!(size.height, 0);

    if config.modes {
        println!("Available present modes:");
        for (i, mode) in surface_caps.present_modes.iter().enumerate() {
            println!("- [{i}] {:?}", mode);
        }

        ExitCode::SUCCESS.exit_process();
    }

    let present_mode = if let Some(idx) = config.mode
        { surface_caps.present_modes[idx] } else
        { wgpu::PresentMode::AutoVsync    };
    
    info!("Present mode: {:?}", present_mode);
    let surface_config = wgpu::SurfaceConfiguration {
        usage        : wgpu::TextureUsages::RENDER_ATTACHMENT,
        format       : surface_format,
        width        : size.width,
        height       : size.height,
        present_mode : present_mode,
        alpha_mode   : surface_caps.alpha_modes[0],
        view_formats : vec![],
    };

    surface.configure(&device, &surface_config);

    let scale = if let Some(scale) = config.scale
        { scale                 } else
        { window.scale_factor() };

    let layout = Layout::new(&device);

    return Graphics {
        device,
        surface,
        queue,
        format: surface_format,
        config: surface_config,
        size,
        scale,

        layout,
    };
}