use std::{fmt::Write as _, path::Path};

use apex_framework::graphics::{
  color::Color,
  drawable::Drawable as _,
  video_exporter::{EncodingPreset, VideoExporter, VideoExporterConfig},
};
use egui::Widget as _;
use pollster::FutureExt as _;
use tap::Tap as _;

use crate::client::{
  gameplay::{beatmap::Beatmap, beatmap_cache::BeatmapCache},
  graphics::{
    taiko_renderer::taiko_renderer::{TaikoRenderer, TaikoRendererConfig},
    taiko_video_exporter::TaikoVideoExporterCallback,
  },
};

pub struct RecordingPanelView {
  pub is_open: bool,

  buffer: String,
}

impl RecordingPanelView {
  pub fn new() -> Self {
    return Self { is_open: false, buffer: String::new() };
  }

  pub fn prepare(
    &mut self,
    ctx: &egui::Context,
    adapter: &wgpu::Adapter,
    format: wgpu::TextureFormat,
    path: &Path,
    cache: &BeatmapCache,
    cfg: &mut VideoExporterConfig,
  ) {
    let mut is_open = self.is_open;

    egui::Window::new("Recording") //
      .resizable(false)
      .collapsible(false)
      .open(&mut is_open)
      .show(ctx, |ui| {
        egui::Grid::new("recording_grid") //
          .num_columns(2)
          .spacing([40.0, 4.0])
          .striped(false)
          .show(ui, |ui| {
            ui.label("Current Beatmap");
            ui.add(
              egui::TextEdit::singleline(&mut path.to_str().unwrap())
                .hint_text("/Users/polina4096/Videos/my_video.mp4"),
            );
            ui.end_row();

            ui.label("Output Path");
            ui.add(egui::TextEdit::singleline(&mut cfg.output_path).hint_text("/Users/polina4096/Videos/my_video.mp4"));
            ui.end_row();

            ui.label("Display Mode");
            ui.horizontal(|ui| {
              ui.set_width(212.0);
              egui_extras::StripBuilder::new(ui)
                .size(egui_extras::Size::relative(0.375))
                .size(egui_extras::Size::remainder())
                .size(egui_extras::Size::relative(0.375))
                .size(egui_extras::Size::remainder())
                .size(egui_extras::Size::relative(0.25))
                .size(egui_extras::Size::remainder())
                .horizontal(|mut strip| {
                  strip.cell(|ui| {
                    self.buffer.clear();
                    write!(self.buffer, "{}", cfg.display_mode.width).unwrap();
                    if egui::TextEdit::singleline(&mut self.buffer)
                      .horizontal_align(egui::Align::Center)
                      .hint_text("1920")
                      .ui(ui)
                      .changed()
                    {
                      if let Ok(width) = self.buffer.parse() {
                        cfg.display_mode.width = width;
                      };
                    };
                  });

                  strip.cell(|ui| {
                    ui.label("x");
                  });

                  strip.cell(|ui| {
                    self.buffer.clear();
                    write!(self.buffer, "{}", cfg.display_mode.height).unwrap();
                    if egui::TextEdit::singleline(&mut self.buffer)
                      .horizontal_align(egui::Align::Center)
                      .hint_text("1080")
                      .ui(ui)
                      .changed()
                    {
                      if let Ok(height) = self.buffer.parse() {
                        cfg.display_mode.height = height;
                      };
                    };
                  });

                  strip.cell(|ui| {
                    ui.label("@");
                  });

                  strip.cell(|ui| {
                    self.buffer.clear();
                    write!(self.buffer, "{}", cfg.display_mode.framerate).unwrap();
                    if egui::TextEdit::singleline(&mut self.buffer)
                      .horizontal_align(egui::Align::Center)
                      .hint_text("120")
                      .ui(ui)
                      .changed()
                    {
                      if let Ok(framerate) = self.buffer.parse() {
                        cfg.display_mode.framerate = framerate;
                      };
                    };
                  });

                  strip.cell(|ui| {
                    ui.label("hz");
                  });
                });
            });

            ui.end_row();

            ui.label("Preset");
            ui.horizontal(|ui| {
              egui::ComboBox::from_id_source("encoding_preset").selected_text(cfg.preset.to_string()).show_ui(
                ui,
                |ui| {
                  ui.selectable_value(&mut cfg.preset, EncodingPreset::Ultrafast, "ultrafast");
                  ui.selectable_value(&mut cfg.preset, EncodingPreset::Superfast, "superfast");
                  ui.selectable_value(&mut cfg.preset, EncodingPreset::Veryfast, "veryfast");
                  ui.selectable_value(&mut cfg.preset, EncodingPreset::Faster, "faster");
                  ui.selectable_value(&mut cfg.preset, EncodingPreset::Fast, "fast");
                  ui.selectable_value(&mut cfg.preset, EncodingPreset::Medium, "medium");
                  ui.selectable_value(&mut cfg.preset, EncodingPreset::Slow, "slow");
                  ui.selectable_value(&mut cfg.preset, EncodingPreset::Slower, "slower");
                  ui.selectable_value(&mut cfg.preset, EncodingPreset::Veryslow, "veryslow");
                },
              );

              ui.add(egui::Slider::new(&mut cfg.crf_bitrate, 0 ..= 51).text("CRF").clamp_to_range(true));
            });

            ui.end_row();
          });

        ui.separator();

        ui.horizontal(|ui| {
          if ui.button("⏺ Record").clicked() {
            let data = std::fs::read_to_string(path).unwrap();
            let beatmap = Beatmap::parse(data, path.to_owned());
            let info = cache.get(beatmap.hash()).unwrap();
            let preview_time = info.preview_time;
            let audio_path = info.audio_path.clone();
            let path = path.to_owned();
            let cfg = cfg.clone();

            let (device, queue) = adapter
              .request_device(
                &wgpu::DeviceDescriptor {
                  label: None,

                  required_features: wgpu::Features::empty(),

                  // WebGL doesn't support all of wgpu's features, so if
                  // we're building for the web we'll have to disable some.
                  required_limits: {
                    if cfg!(target_arch = "wasm32") {
                      wgpu::Limits::downlevel_webgl2_defaults().tap_mut(|limits| {
                        limits.max_texture_dimension_2d = 8192;
                        limits.max_bind_groups = 8;
                      })
                    } else {
                      wgpu::Limits::default().tap_mut(|limits| {
                        limits.max_bind_groups = 8;
                      })
                    }
                  },
                  memory_hints: wgpu::MemoryHints::Performance,
                },
                None, // Trace path
              )
              .block_on()
              .expect("Failed to retrieve a device");

            std::thread::spawn(move || {
              let mut renderer = TaikoRenderer::new(
                &device,
                &queue,
                format,
                TaikoRendererConfig {
                  width: cfg.display_mode.width as f32,
                  height: cfg.display_mode.height as f32,
                  scale_factor: 2.0,
                  gameplay_scale: 0.85,
                  conveyor_zoom: 0.235,
                  hit_position_x: 128.0,
                  hit_position_y: 256.0,
                  don: Color::new(0.92, 0.00, 0.27, 1.00),
                  kat: Color::new(0.00, 0.47, 0.67, 1.00),
                  hit_animation_height: 12.5,
                },
              );

              renderer.load_beatmap(&device, beatmap);
              renderer.set_hit_all(&queue);

              renderer.resize(
                &device,
                &queue,
                cfg.display_mode.width as f32 / 2.0,
                cfg.display_mode.height as f32 / 2.0,
              );

              let exporter = VideoExporter::new(&device, format, &cfg);

              exporter.export(
                &cfg,
                &device,
                &queue,
                path.parent().unwrap().canonicalize().unwrap().join(audio_path),
                preview_time,
                0 .. (cfg.display_mode.framerate as i32 * 10),
                TaikoVideoExporterCallback::new(&mut renderer, preview_time),
              );
            });
          };
          if ui.button("🖹 Logs").clicked() {};
        });
      });

    self.is_open = is_open;
  }
}
