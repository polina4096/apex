use apex_framework::input::Input;
use egui::Widget;
use tap::Tap as _;

use crate::client::{
  action::ClientAction,
  settings::{Settings, SettingsProxy},
};

pub struct VolumeScreen {}

impl VolumeScreen {
  pub fn new() -> Self {
    return Self {};
  }

  pub fn prepare(
    &mut self,
    ctx: &egui::Context,
    input: &Input<ClientAction>,
    proxy: &mut impl SettingsProxy,
    settings: &mut Settings,
  ) {
    if !input.state.modifiers.alt_key() {
      return;
    }

    egui::Window::new("Volume_Master")
      .frame(egui::Frame::none())
      .title_bar(false)
      .resizable(false)
      .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -16.0))
      .show(ctx, |ui| {
        ui.vertical(|ui| {
          let mut master_volume = settings.audio.volume.master_volume();

          {
            ui.add_space(3.0);

            let rect = ui.available_rect_before_wrap().tap_mut(|rect| {
              *rect = rect.translate(egui::vec2(-4.0, -6.0));
              rect.set_width(41.0);
              rect.set_height(24.5);
            });

            ui.painter().rect_filled(
              rect,
              egui::Rounding::same(6.0),
              egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220),
            );

            let rect = ui.available_rect_before_wrap();
            let text = ui.painter().layout_no_wrap(
              format!("{:.0}%", master_volume * 100.0),
              egui::FontId::proportional(12.0),
              egui::Color32::PLACEHOLDER,
            );

            ui.add_space(text.size().y);
            let offset = 17.0 - text.size().x / 2.0;
            let pos = egui::pos2(rect.min.x + offset, rect.min.y);
            ui.painter().galley(pos, text, ui.style().visuals.text_color());
          }

          ui.add_space(8.0);

          ui.horizontal(|ui| {
            ui.add_space(3.0);

            egui::Frame::window(ui.style()).show(ui, |ui| {
              ui.vertical(|ui| {
                ui.add_space(2.0);

                let slider = egui::Slider::new(&mut master_volume, 0.0 ..= 1.0) //
                  .vertical()
                  .show_value(false)
                  .trailing_fill(true);

                if slider.ui(ui).changed() {
                  settings.audio.volume.set_master_volume(master_volume, proxy);
                }

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                  ui.add_space(4.0);
                  ui.label("M")
                });
              });
            });
          });
        });
      });

    let delta = ctx.input(|x| x.smooth_scroll_delta.y);

    if delta != 0.0 {
      let speed = 0.001;
      let new_volume = settings.audio.volume.master_volume() + delta * speed;
      settings.audio.volume.set_master_volume(new_volume.clamp(0.0, 1.0), proxy);
      ctx.input_mut(|x| x.smooth_scroll_delta = egui::Vec2::ZERO);
    }
  }
}
