use egui::Widget as _;

use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    state::{
      graphics_state::{FrameLimiterOptions, PresentModeOptions, RenderingBackend, WgpuBackend},
      AppState,
    },
  },
  core::core::Core,
};

use super::GameSettingsView;

impl GameSettingsView {
  pub(super) fn general_tab(&mut self, ui: &mut egui::Ui, core: &Core<Client>, state: &mut AppState) {
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
        self.gameplay_category(&mut body, text_height, state);
        self.graphics_category(&mut body, text_height, core, state);
        self.taiko_category(&mut body, text_height, state);
      });
  }

  fn graphics_category(
    &mut self,
    body: &mut egui_extras::TableBody,
    text_height: f32,
    core: &Core<Client>,
    state: &mut AppState,
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
        let selected = &mut state.graphics.present_mode;
        let available_width = ui.available_width() - 192.0;

        egui::ComboBox::new("present_mode", "")
          .selected_text(format!("{:?}", selected))
          .width(available_width)
          .show_ui(ui, |ui| {
            if ui.selectable_value(selected, PresentModeOptions::VSync, "VSync").changed()
              || ui.selectable_value(selected, PresentModeOptions::Immediate, "Immediate").changed()
            {
              core.reconfigure_surface_texture();
            };
          });
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Frame Limiter");
      });

      row.col(|ui| {
        let selected = &mut state.graphics.frame_limiter;
        let available_width = ui.available_width() - 192.0;

        egui::ComboBox::new("frame_limiter", "")
          .selected_text(format!("{:?}", selected))
          .width(available_width)
          .show_ui(ui, |ui| {
            if { false }
              || ui.selectable_value(selected, FrameLimiterOptions::Custom(240), "240 fps").changed()
              || ui.selectable_value(selected, FrameLimiterOptions::Custom(480), "480 fps").changed()
              || ui.selectable_value(selected, FrameLimiterOptions::Custom(960), "960 fps").changed()
              || ui.selectable_value(selected, FrameLimiterOptions::Unlimited, "Unlimited").changed()
              || ui.selectable_value(selected, FrameLimiterOptions::DisplayLink, "Display Link").changed()
            {
              core.update_frame_limiter_configuration();
            };
          });
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Renderer Backend");
      });

      row.col(|ui| {
        let selected = &mut state.graphics.rendering_backend;
        let available_width = ui.available_width() - 192.0;

        egui::ComboBox::new("renderer_backend", "")
          .selected_text(format!("{:?}", selected))
          .width(available_width)
          .show_ui(ui, |ui| {
            if { false }
              || ui.selectable_value(selected, RenderingBackend::Wgpu(WgpuBackend::Auto), "Auto").changed()
              || ui.selectable_value(selected, RenderingBackend::Wgpu(WgpuBackend::Vulkan), "Vulkan").changed()
              || ui.selectable_value(selected, RenderingBackend::Wgpu(WgpuBackend::Metal), "Metal").changed()
              || ui.selectable_value(selected, RenderingBackend::Wgpu(WgpuBackend::Dx12), "DirectX 12").changed()
              || ui.selectable_value(selected, RenderingBackend::Wgpu(WgpuBackend::Gl), "OpenGL").changed()
              || ui.selectable_value(selected, RenderingBackend::Wgpu(WgpuBackend::WebGpu), "WebGPU").changed()
            {
              core.recreate_graphics_context();
            };
          });
      });
    });
  }

  fn gameplay_category(&mut self, body: &mut egui_extras::TableBody, text_height: f32, state: &mut AppState) {
    body.row(text_height + 8.0, |mut row| {
      row.col(|ui| {
        let text = egui::RichText::new("Gameplay").strong().heading();
        egui::Label::new(text).ui(ui);
      });

      row.col(|_| {});
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Audio Offset");
      });

      row.col(|ui| {
        egui::Slider::new(&mut state.gameplay.audio_offset, -100 ..= 100).clamp_to_range(false).ui(ui);
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Lead in");
      });

      row.col(|ui| {
        egui::Slider::new(&mut state.gameplay.lead_in, -100 ..= 100).clamp_to_range(false).ui(ui);
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Lead out");
      });

      row.col(|ui| {
        egui::Slider::new(&mut state.gameplay.lead_out, -100 ..= 100).clamp_to_range(false).ui(ui);
      });
    });
  }

  fn taiko_category(&mut self, body: &mut egui_extras::TableBody, text_height: f32, state: &mut AppState) {
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
        let slider = egui::Slider::new(&mut state.taiko.zoom, 0.0 ..= 1.0).step_by(0.001).ui(ui);

        if slider.changed() {
          self.event_bus.send(ClientEvent::RebuildTaikoRendererInstances);
        }
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Scale");
      });

      row.col(|ui| {
        egui::Slider::new(&mut state.taiko.scale, 0.0 ..= 2.0).ui(ui);
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Hit position x");
      });

      row.col(|ui| {
        egui::DragValue::new(&mut state.taiko.hit_position_x).ui(ui);
      });
    });

    body.row(text_height, |mut row| {
      row.col(|ui| {
        ui.label("Hit position y");
      });

      row.col(|ui| {
        egui::DragValue::new(&mut state.taiko.hit_position_y).ui(ui);
      });
    });

    body.row(text_height, |mut row| {
      use egui::color_picker::{color_edit_button_rgba, Alpha};

      row.col(|ui| {
        ui.label("Don color");
      });

      row.col(|ui| {
        let (r, g, b, a) = state.taiko.don_color.as_rgba();
        let mut color = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
        if color_edit_button_rgba(ui, &mut color, Alpha::Opaque).changed() {
          state.taiko.don_color = color.into();
          self.event_bus.send(ClientEvent::RebuildTaikoRendererInstances);
        }
      });
    });

    body.row(text_height, |mut row| {
      use egui::color_picker::{color_edit_button_rgba, Alpha};

      row.col(|ui| {
        ui.label("Kat color");
      });

      row.col(|ui| {
        let (r, g, b, a) = state.taiko.kat_color.as_rgba();
        let mut color = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
        if color_edit_button_rgba(ui, &mut color, Alpha::Opaque).changed() {
          state.taiko.kat_color = color.into();
          self.event_bus.send(ClientEvent::RebuildTaikoRendererInstances);
        }
      });
    });
  }
}
