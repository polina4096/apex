use log::info;
use tap::Tap;
use wgpu::{Backends, InstanceFlags, PowerPreference, SurfaceTargetUnsafe};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Graphics {
  pub device  : wgpu::Device,
  pub surface : wgpu::Surface<'static>,
  pub queue   : wgpu::Queue,
  pub format  : wgpu::TextureFormat,
  pub config  : wgpu::SurfaceConfiguration,

  pub size    : PhysicalSize<u32>,
  pub scale   : f64,
}

impl Graphics {
  pub async fn new(window: &Window) -> Graphics {
    let instance = wgpu::Instance::new(
      wgpu::InstanceDescriptor {
        flags                : InstanceFlags::empty(),
        backends             : Backends::all().tap_mut(|x| x.remove(wgpu::Backends::BROWSER_WEBGPU)),
        dx12_shader_compiler : Default::default(),
        gles_minor_version   : wgpu::Gles3MinorVersion::Automatic,
      }
    );

    // # Safety
    // The surface needs to live as long as the window that created it.
    // State owns the window so this should be safe.
    let target = unsafe { SurfaceTargetUnsafe::from_window(window) }.expect("Failed to create suface target");
    let surface = unsafe { instance.create_surface_unsafe(target) }.expect("Failed to create a surface");

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference       : PowerPreference::HighPerformance,
      compatible_surface     : Some(&surface),
      force_fallback_adapter : false,
    }).await.expect("Failed to retrive an adapter");

    let info = adapter.get_info();
    info!("Selected GPU: {} | ({:?})", info.name, info.device_type);
    info!("Selected backend: {:?}", info.backend);
    info!("Driver: {} | {}", info.driver, info.driver_info);

    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        label: None,

        required_features: wgpu::Features::empty(),

        // WebGL doesn't support all of wgpu's features, so if
        // we're building for the web we'll have to disable some.
        required_limits: {
          if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
              .tap_mut(|limits| {
                limits.max_texture_dimension_2d = 8192;
                limits.max_bind_groups = 8;
              })
          } else {
            wgpu::Limits::default()
              .tap_mut(|limits| {
                limits.max_bind_groups = 8;
              })
          }
        },
      },
      None, // Trace path
    ).await.expect("Failed to retrieve a device");

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter().copied()
      .find(|f| f.is_srgb())
      .unwrap_or(surface_caps.formats[0]);

    info!("Surface format: {:?}", surface_format);

    cfg_if::cfg_if! {
      if #[cfg(target_arch = "wasm32")] {
        use winit::platform::web::WindowExtWebSys;
        let web_window = web_sys::window().expect("Failed to get web window object");
        let size: PhysicalSize<u32> = winit::dpi::LogicalSize::new(
          web_window.inner_width().unwrap().as_f64().unwrap(),
          web_window.inner_height().unwrap().as_f64().unwrap(),
        ).to_physical(window.scale_factor());
      } else {
        let size = window.inner_size();
      }
    }

    assert_ne!(size.width, 0);
    assert_ne!(size.height, 0);

    let present_mode = wgpu::PresentMode::AutoVsync;
    info!("Present mode: {:?}", present_mode);

    let surface_config = wgpu::SurfaceConfiguration {
      usage        : wgpu::TextureUsages::RENDER_ATTACHMENT,
      format       : surface_format,
      width        : size.width,
      height       : size.height,
      present_mode : present_mode,
      alpha_mode   : surface_caps.alpha_modes[0],
      view_formats : vec![],

      desired_maximum_frame_latency: 1,
    };

    surface.configure(&device, &surface_config);

    let scale = window.scale_factor();

    return Graphics {
      device,
      surface,
      queue,
      format: surface_format,
      config: surface_config,

      size,
      scale,
    };
  }
}