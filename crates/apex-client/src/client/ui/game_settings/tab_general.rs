use egui::Widget as _;

use crate::client::{
  graphics::{FrameLimiterOptions, PresentModeOptions, RenderingBackend, WgpuBackend},
  settings::{Settings, SettingsProxy},
};

use super::GameSettingsView;

impl GameSettingsView {
  pub(super) fn general_tab(&mut self, ui: &mut egui::Ui, settings: &mut Settings, proxy: &mut impl SettingsProxy) {
    use egui_extras::{Column, TableBuilder};

    let text_height = egui::TextStyle::Body.resolve(ui.style()).size.max(ui.spacing().interact_size.y);
    let available_width = ui.available_width() - 192.0;
    ui.style_mut().spacing.slider_width = available_width;

    TableBuilder::new(ui)
      .striped(true)
      .resizable(false)
      .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
      .column(Column::exact(128.0))
      .column(Column::remainder())
      .body(|mut body| {
        self.audio_category(&mut body, text_height, settings, proxy);
        self.graphics_category(&mut body, text_height, settings, proxy);
        self.profile_category(&mut body, text_height, settings, proxy);
        self.gameplay_category(&mut body, text_height, settings, proxy);
        self.taiko_category(&mut body, text_height, settings, proxy);
      });
  }

  fn audio_category(
    &mut self,
    body: &mut egui_extras::TableBody,
    text_height: f32,
    settings: &mut Settings,
    proxy: &mut impl SettingsProxy,
  ) {
    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        ui.label("Master Volume");
      });

      row.col(|ui| {
        let mut value = settings.audio.master_volume();
        if egui::Slider::new(&mut value, 0.0 ..= 1.0).clamp_to_range(true).ui(ui).changed() {
          settings.audio.set_master_volume(value, proxy);
        }
      });
    });

    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        ui.label("Music Volume");
      });

      row.col(|ui| {
        let mut value = settings.audio.music_volume();
        if egui::Slider::new(&mut value, 0.0 ..= 1.0).clamp_to_range(true).ui(ui).changed() {
          settings.audio.set_music_volume(value, proxy);
        }
      });
    });

    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        ui.label("Effect Volume");
      });

      row.col(|ui| {
        let mut value = settings.audio.effect_volume();
        if egui::Slider::new(&mut value, 0.0 ..= 1.0).clamp_to_range(true).ui(ui).changed() {
          settings.audio.set_effect_volume(value, proxy);
        }
      });
    });
  }

  fn graphics_category(
    &mut self,
    body: &mut egui_extras::TableBody,
    text_height: f32,
    settings: &mut Settings,
    proxy: &mut impl SettingsProxy,
  ) {
    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        let text = egui::RichText::new("Graphics").strong().heading();
        egui::Label::new(text).ui(ui);
      });

      row.col(|_| {});
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Present Mode");
      });

      row.col(|ui| {
        let available_width = ui.available_width() - 192.0;

        let mut selected = settings.graphics.present_mode();
        egui::ComboBox::new("present_mode", "")
          .selected_text(format!("{:?}", selected))
          .width(available_width)
          .show_ui(ui, |ui| {
            if { false }
              || ui.selectable_value(&mut selected, PresentModeOptions::VSync, "VSync").changed()
              || ui.selectable_value(&mut selected, PresentModeOptions::Immediate, "Immediate").changed()
            {
              settings.graphics.set_present_mode(selected, proxy);
            }
          });
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Frame Limiter");
      });

      row.col(|ui| {
        let available_width = ui.available_width() - 192.0;

        let mut selected = settings.graphics.frame_limiter();
        egui::ComboBox::new("frame_limiter", "")
          .selected_text(format!("{:?}", selected))
          .width(available_width)
          .show_ui(ui, |ui| {
            if { false }
              || ui.selectable_value(&mut selected, FrameLimiterOptions::Custom(240), "240 fps").changed()
              || ui.selectable_value(&mut selected, FrameLimiterOptions::Custom(480), "480 fps").changed()
              || ui.selectable_value(&mut selected, FrameLimiterOptions::Custom(960), "960 fps").changed()
              || ui.selectable_value(&mut selected, FrameLimiterOptions::Unlimited, "Unlimited").changed()
              || ui.selectable_value(&mut selected, FrameLimiterOptions::DisplayLink, "Display Link").changed()
            {
              settings.graphics.set_frame_limiter(selected, proxy);
            }
          });
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Renderer Backend");
      });

      row.col(|ui| {
        let available_width = ui.available_width() - 192.0;

        let mut selected = settings.graphics.rendering_backend();
        egui::ComboBox::new("renderer_backend", "")
          .selected_text(format!("{:?}", selected))
          .width(available_width)
          .show_ui(ui, |ui| {
            if { false }
              || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Auto), "Auto").changed()
              || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Vulkan), "Vulkan").changed()
              || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Metal), "Metal").changed()
              || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Dx12), "DX 12").changed()
              || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::Gl), "OpenGL").changed()
              || ui.selectable_value(&mut selected, RenderingBackend::Wgpu(WgpuBackend::WebGpu), "WebGPU").changed()
            {
              settings.graphics.set_rendering_backend(selected, proxy);
            }
          });
      });
    });

    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        ui.label("Max Frame Latency");
      });

      row.col(|ui| {
        let mut value = settings.graphics.max_frame_latency();
        if egui::DragValue::new(&mut value).range(0 ..= 5).clamp_to_range(false).ui(ui).changed() {
          settings.graphics.set_max_frame_latency(value, proxy);
        }
      });
    });

    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        ui.label("macOS stutter fix");
      });

      row.col(|ui| {
        let mut value = settings.graphics.macos_stutter_fix();
        if egui::Checkbox::without_text(&mut value).ui(ui).changed() {
          let prev = settings.graphics.frame_limiter();
          settings.graphics.set_frame_limiter(FrameLimiterOptions::Unlimited, proxy);
          settings.graphics.set_macos_stutter_fix(value, proxy);

          // This is a horrible workaround so to successfully restart Display Link,
          // reset the frame limiter triggering the Display Link reinitialization
          settings.graphics.set_frame_limiter(prev, proxy);
          settings.graphics.set_macos_stutter_fix(value, proxy);
        }
      });
    });
  }

  fn profile_category(
    &mut self,
    body: &mut egui_extras::TableBody,
    text_height: f32,
    settings: &mut Settings,
    proxy: &mut impl SettingsProxy,
  ) {
    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        let text = egui::RichText::new("Profile").strong().heading();
        egui::Label::new(text).ui(ui);
      });

      row.col(|_| {});
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Username");
      });

      row.col(|ui| {
        self.buffer.clone_from(settings.profile.borrowed_username());
        if egui::TextEdit::singleline(&mut self.buffer).desired_width(256.0).ui(ui).changed() {
          settings.profile.set_borrowed_username(&self.buffer, proxy);
        }

        self.buffer.clear();
      });
    });
  }

  fn gameplay_category(
    &mut self,
    body: &mut egui_extras::TableBody,
    text_height: f32,
    settings: &mut Settings,
    proxy: &mut impl SettingsProxy,
  ) {
    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        let text = egui::RichText::new("Gameplay").strong().heading();
        egui::Label::new(text).ui(ui);
      });

      row.col(|_| {});
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Universal Offset");
      });

      row.col(|ui| {
        let mut value = settings.gameplay.universal_offset();
        if egui::Slider::new(&mut value, -500 ..= 500).clamp_to_range(false).ui(ui).changed() {
          settings.gameplay.set_universal_offset(value, proxy);
        }
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Lead In");
      });

      row.col(|ui| {
        let mut value = settings.gameplay.lead_in();
        if egui::Slider::new(&mut value, 0 ..= 100).clamp_to_range(false).ui(ui).changed() {
          settings.gameplay.set_lead_in(value, proxy);
        }
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Lead Out");
      });

      row.col(|ui| {
        let mut value = settings.gameplay.lead_out();
        if egui::Slider::new(&mut value, 0 ..= 100).clamp_to_range(false).ui(ui).changed() {
          settings.gameplay.set_lead_out(value, proxy);
        }
      });
    });
  }

  fn taiko_category(
    &mut self,
    body: &mut egui_extras::TableBody,
    text_height: f32,
    settings: &mut Settings,
    proxy: &mut impl SettingsProxy,
  ) {
    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        let text = egui::RichText::new("Taiko").strong().heading();
        egui::Label::new(text).ui(ui);
      });

      row.col(|_| {});
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Zoom");
      });

      row.col(|ui| {
        let mut value = settings.taiko.zoom();
        if egui::Slider::new(&mut value, 0.0 ..= 1.0).step_by(0.001).ui(ui).changed() {
          settings.taiko.set_zoom(value, proxy);
        }
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Scale");
      });

      row.col(|ui| {
        let mut value = settings.taiko.scale();
        if egui::Slider::new(&mut value, 0.0 ..= 2.0).ui(ui).changed() {
          settings.taiko.set_scale(value, proxy);
        }
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Hit position x");
      });

      row.col(|ui| {
        let mut value = settings.taiko.hit_position_x();
        if egui::DragValue::new(&mut value).ui(ui).changed() {
          settings.taiko.set_hit_position_x(value, proxy);
        }
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Hit position y");
      });

      row.col(|ui| {
        let mut value = settings.taiko.hit_position_y();
        if egui::DragValue::new(&mut value).ui(ui).changed() {
          settings.taiko.set_hit_position_y(value, proxy);
        }
      });
    });

    body.row(text_height, |mut row| {
      use egui::color_picker::{color_edit_button_rgba, Alpha};

      row.col(|ui| {
        ui.label("Don color");
      });

      row.col(|ui| {
        let (r, g, b, a) = settings.taiko.don_color().as_rgba();
        let mut color = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
        if color_edit_button_rgba(ui, &mut color, Alpha::Opaque).changed() {
          settings.taiko.set_don_color(color.into(), proxy);
        }
      });
    });

    body.row(text_height, |mut row| {
      use egui::color_picker::{color_edit_button_rgba, Alpha};

      row.col(|ui| {
        ui.label("Kat color");
      });

      row.col(|ui| {
        let (r, g, b, a) = settings.taiko.kat_color().as_rgba();
        let mut color = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
        if color_edit_button_rgba(ui, &mut color, Alpha::Opaque).changed() {
          settings.taiko.set_kat_color(color.into(), proxy);
        }
      });
    });

    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        ui.label("Hit animation");
      });

      row.col(|ui| {
        let mut value = settings.taiko.hit_animation();
        if egui::Checkbox::without_text(&mut value).ui(ui).changed() {
          settings.taiko.set_hit_animation(value, proxy);
        }
      });
    });
  }
}
