pub mod proxy;

use apex_framework::{data::persistent::Persistent, graphics::color::Color, SettingsGroup, SettingsStruct};
use egui::Widget as _;
use macro_rules_attribute::derive;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use tap::Pipe;

use crate::client::graphics::{FrameLimiterOptions, PresentModeOptions, RenderingBackend, WgpuBackend};

use super::score::score::Score;

#[derive(SettingsStruct!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Settings {
  pub profile: ProfileSettings,
  pub audio: AudioSettings,
  pub graphics: GraphicsSettings,
  pub gameplay: GameplaySettings,
  pub interface: InterfaceSettings,
  pub taiko: TaikoSettings,
}

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct ProfileSettings {
  /// Player username
  #[default(String::from(Score::DEFAULT_USERNAME))]
  username: String,
}

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct AudioSettings {
  /// Master volume
  #[default = 0.25]
  #[custom(ui(range = 0.0 ..= 1.0))]
  master_volume: f32,

  /// Music volume
  #[default = 1.0]
  #[custom(ui(range = 0.0 ..= 1.0))]
  music_volume: f32,

  /// Effect volume
  #[default = 1.0]
  #[custom(ui(range = 0.0 ..= 1.0))]
  effects_volume: f32,
}

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct GraphicsSettings {
  /// Controls the frame pacing
  #[default(Default::default())]
  frame_limiter: FrameLimiterOptions,

  /// Graphics API presentation mode
  #[default(PresentModeOptions::VSync)]
  present_mode: PresentModeOptions,

  /// Rendering backend to use
  #[default(RenderingBackend::Wgpu(WgpuBackend::Auto))]
  rendering_backend: RenderingBackend,

  /// Hints the GPU how many frames to buffer
  #[default = 2]
  #[custom(ui(range = 0 ..= 5))]
  max_frame_latency: usize,

  /// Fixes massive macOS game stutter when alt-tabbing occluded window
  #[default = true]
  macos_stutter_fix: bool,
}

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct GameplaySettings {
  /// Offset of the audio in milliseconds
  #[default = 0]
  #[custom(ui(range = -500 ..= 500))]
  universal_offset: i64,

  /// Additional time before the first note
  #[default = 1000]
  #[custom(ui(range = 0 ..= 5000))]
  lead_in: u64,

  /// Additional time after the last note
  #[default = 1000]
  #[custom(ui(range = 0 ..= 5000))]
  lead_out: u64,

  /// Additional time before a break overlay is show
  #[default = 1000]
  #[custom(ui(range = 0 ..= 5000))]
  break_leniency_start: u64,

  /// Break overlay is hidden this much earlier
  #[default = 1000]
  #[custom(ui(range = 0 ..= 5000))]
  break_leniency_end: u64,
}

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct InterfaceSettings {
  /// Total width of the hit delta bar
  #[default = 128.0]
  #[custom(ui(clamp = false, slider = false))]
  delta_bar_width: f32,

  /// Height of the hit delta bar mark
  #[default = 24.0]
  #[custom(ui(clamp = false, slider = false))]
  delta_bar_height: f32,

  /// Opacity of the hit delta bar areas
  #[default = 0.05]
  #[custom(ui(range = 0.0 ..= 1.0))]
  delta_bar_opacity: f32,

  /// Width of the hit delta marker
  #[default = 2.0]
  #[custom(ui(clamp = false, slider = false))]
  delta_marker_width: f32,

  /// Height of the hit delta marker
  #[default = 16.0]
  #[custom(ui(clamp = false, slider = false))]
  delta_marker_height: f32,

  /// Opacity of the hit delta marker
  #[default = 0.25]
  #[custom(ui(range = 0.0 ..= 1.0))]
  delta_marker_opacity: f32,

  /// Duration for which the hit delta marker is shown in seconds
  #[default = 1.0]
  #[custom(ui(range = 0.0 ..= 10.0))]
  delta_marker_duration: f32,

  /// Duration it takes for the hit delta marker to fade in or out in seconds
  #[default = 0.2]
  #[custom(ui(range = 0.0 ..= 10.0))]
  delta_marker_fade: f32,
}

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct TaikoSettings {
  /// Hit object distance multiplier
  #[default = 0.215]
  #[custom(ui(range = 0.0 ..= 2.0))]
  zoom: f64,

  /// Gameplay scale
  #[default = 0.85]
  #[custom(ui(range = 0.0 ..= 10.0))]
  scale: f64,

  /// Hit position X
  #[default = 256.0]
  #[custom(ui(clamp = false, slider = false))]
  hit_position_x: f32,

  /// Hit position Y
  #[default = 192.0]
  #[custom(ui(clamp = false, slider = false))]
  hit_position_y: f32,

  /// Color of the don hit object
  #[default(Color::new(0.92, 0.00, 0.27, 1.00))]
  don_color: Color,

  /// Color of the kat hit object
  #[default(Color::new(0.00, 0.47, 0.67, 1.00))]
  kat_color: Color,

  /// Hit animation
  #[default = true]
  hit_animation: bool,
}

mod settings_ui {
  use super::*;

  use std::cell::RefCell;

  use apex_framework::data::settings::NumericOpts;

  macro_rules! make_numeric_ui {
    (
      $($ty:ty)+
    ) => {
      paste::paste! {
        $(
          #[allow(dead_code)]
          pub fn [<ui_ $ty:snake>](ui: &mut egui::Ui, value: &$ty, opts: NumericOpts<$ty>) -> Option<$ty> {
            let mut new_value = None;

            let mut value = *value;
            let widget = if opts.slider {
              egui::Slider::new(&mut value, opts.range)
                .step_by(opts.step)
                .clamp_to_range(opts.clamp)
                .pipe(|slider| if let Some(decimals) = opts.precision {
                  slider.max_decimals(decimals)
                } else {
                  slider
                })
                .ui(ui)
            } else {
              egui::DragValue::new(&mut value)
                .speed(if opts.step == 0.0 { 1.0 } else { opts.step })
                .range(opts.range)
                .clamp_to_range(opts.clamp)
                .max_decimals_opt(opts.precision)
                .ui(ui)
            };

            if widget.changed() {
              new_value = Some(value);
            }

            return new_value;
          }
        )+
      }
    };
  }

  make_numeric_ui!(
    i8 i16 i32 i64
    u8 u16 u32 u64
    isize usize
    f32 f64
  );

  pub fn ui_bool(ui: &mut egui::Ui, value: &bool) -> Option<bool> {
    let mut new_value = None;

    let mut value = *value;
    if egui::Checkbox::without_text(&mut value).ui(ui).changed() {
      new_value = Some(value);
    }

    return new_value;
  }

  pub fn ui_string(ui: &mut egui::Ui, value: &String) -> Option<String> {
    let mut new_value = None;

    thread_local! {
      static BUFFER: RefCell<String> = const { RefCell::new(String::new()) };
    }

    BUFFER.with_borrow_mut(|x| {
      x.clone_from(value);
      if egui::TextEdit::singleline(x).ui(ui).changed() {
        new_value = Some(x.clone());
      }
    });

    return new_value;
  }

  pub fn ui_color(ui: &mut egui::Ui, value: &Color) -> Option<Color> {
    let mut new_value = None;

    use egui::color_picker::{color_edit_button_rgba, Alpha};
    let (mut r, mut g, mut b, mut a) = value.as_rgba();
    let mut color = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
    if color_edit_button_rgba(ui, &mut color, Alpha::Opaque).changed() {
      new_value = Some(color.into());
    }

    if { false }
      | egui::DragValue::new(&mut r).speed(0.0025).fixed_decimals(2).range(0.0 ..= 1.0).ui(ui).changed()
      | egui::DragValue::new(&mut g).speed(0.0025).fixed_decimals(2).range(0.0 ..= 1.0).ui(ui).changed()
      | egui::DragValue::new(&mut b).speed(0.0025).fixed_decimals(2).range(0.0 ..= 1.0).ui(ui).changed()
      | egui::DragValue::new(&mut a).speed(0.0025).fixed_decimals(2).range(0.0 ..= 1.0).ui(ui).changed()
    {
      new_value = Some(Color::new(r, g, b, a));
    }

    return new_value;
  }

  pub fn ui_rendering_backend(ui: &mut egui::Ui, value: &RenderingBackend) -> Option<RenderingBackend> {
    let mut new_value = None;

    let mut selected = *value;
    egui::ComboBox::new("present_mode", "")
      .selected_text(format!("{:?}", selected))
      .width(ui.available_width() - 192.0)
      .show_ui(ui, |ui| {
        if { false }
          || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Auto), "Auto").changed()
          || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Vulkan), "Vulkan").changed()
          || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Metal), "Metal").changed()
          || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Dx12), "DX 12").changed()
          || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Gl), "OpenGL").changed()
          || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::WebGpu), "WebGPU").changed()
        {
          new_value = Some(selected);
        }
      });

    return new_value;
  }

  pub fn ui_frame_limiter_options(ui: &mut egui::Ui, value: &FrameLimiterOptions) -> Option<FrameLimiterOptions> {
    let mut new_value = None;

    let mut selected = *value;
    egui::ComboBox::new("present_mode", "")
      .selected_text(format!("{:?}", selected))
      .width(ui.available_width() - 192.0)
      .show_ui(ui, |ui| {
        if { false }
          || ui.selectable_value(&mut selected, FrameLimiterOptions::Custom(240), "240 fps").changed()
          || ui.selectable_value(&mut selected, FrameLimiterOptions::Custom(480), "480 fps").changed()
          || ui.selectable_value(&mut selected, FrameLimiterOptions::Custom(960), "960 fps").changed()
          || ui.selectable_value(&mut selected, FrameLimiterOptions::Unlimited, "Unlimited").changed()
          || ui.selectable_value(&mut selected, FrameLimiterOptions::DisplayLink, "Display Link").changed()
        {
          new_value = Some(selected);
        }
      });

    return new_value;
  }

  pub fn ui_present_mode_options(ui: &mut egui::Ui, value: &PresentModeOptions) -> Option<PresentModeOptions> {
    let mut new_value = None;

    let mut selected = *value;
    egui::ComboBox::new("present_mode", "")
      .selected_text(format!("{:?}", selected))
      .width(ui.available_width() - 192.0)
      .show_ui(ui, |ui| {
        if { false }
          || ui.selectable_value(&mut selected, PresentModeOptions::VSync, "VSync").changed()
          || ui.selectable_value(&mut selected, PresentModeOptions::Immediate, "Immediate").changed()
        {
          new_value = Some(selected);
        }
      });

    return new_value;
  }
}

impl Persistent for Settings {
  fn load(path: impl AsRef<std::path::Path>) -> Self {
    {
      let path = path.as_ref().canonicalize().unwrap_or(path.as_ref().to_owned());
      log::info!("Loading settings from `{}`", path.display());
    }

    return std::fs::read_to_string(&path)
      .map(|data| {
        return toml::from_str(&data).unwrap_or_else(|e| {
          log::error!("Failed to parse config file, falling back to default config: {}", e);

          return Settings::default();
        });
      })
      .unwrap_or_else(|e| {
        let default = Settings::default();

        match e.kind() {
          std::io::ErrorKind::NotFound => {
            log::warn!("Failed to open config file, file not found. Creating a default config file...");
            let default_data = toml::to_string_pretty(&default).expect("Failed to serialize default config");
            if let Err(e) = std::fs::write(&path, default_data) {
              log::error!("Failed to write default config file: {}", e);
            }
          }

          std::io::ErrorKind::PermissionDenied => {
            log::warn!("Failed to open config file, insufficient permissions. Falling back to default configuration.");
          }

          _ => {
            log::error!("Failed to access config file: {}. Falling back to default configuration.", e);
          }
        }

        return default;
      });
  }

  fn save(&self, path: impl AsRef<std::path::Path>) {
    let data = match toml::to_string_pretty(&self) {
      Ok(data) => data,
      Err(e) => {
        log::error!("Failed to serialize settings: {}", e);
        return;
      }
    };

    if let Err(e) = std::fs::write(&path, data) {
      log::error!("Failed to write settings to file: {}", e);
      return;
    }

    let path = path.as_ref().canonicalize().unwrap_or(path.as_ref().to_owned());
    log::info!("Settings successfully written to `{}`", path.display());
  }
}
