use crate::{
  client::{client::Client, gameplay::beatmap::BreakPoint},
  core::{core::Core, time::time::Time},
};

pub struct BreakOverlayView {}

impl BreakOverlayView {
  pub fn new() -> Self {
    return Self {};
  }

  pub fn prepare(
    &mut self,
    core: &Core<Client>,
    time: Time,
    break_point: &BreakPoint,
    break_leniency_start: Time,
    break_leniency_end: Time,
  ) {
    let frame = egui::Frame::none() //
      .inner_margin(egui::Margin::ZERO);

    egui::CentralPanel::default().frame(frame).show(core.egui_ctx(), |ui| {
      // Start a break this much later to not distract the player
      if time > Time::zero() && time - break_leniency_start < break_point.start {
        return;
      }

      let break_length = break_point.end - break_point.start;

      // End the break this much earlier to not distract the player
      let time = break_length - time - break_leniency_end;

      if time > Time::zero() {
        ui.vertical_centered_justified(|ui| {
          const COUNTDOWN_TEXT_SIZE: f32 = 32.0;
          const SKIP_TEXT_SIZE: f32 = 24.0;

          let text = ui.painter().layout_no_wrap(
            format!("{}", time.to_seconds().ceil() as i32),
            egui::FontId::proportional(COUNTDOWN_TEXT_SIZE),
            egui::Color32::PLACEHOLDER,
          );

          let size = text.size();
          let x = ui.available_width() / 2.0 - size.x / 2.0;
          let y = ui.available_height() / 2.0 - size.y / 2.0;
          ui.painter().galley(egui::pos2(x, y), text, ui.style().visuals.strong_text_color());

          let x = ui.available_width() / 2.0;
          let y = ui.available_height() / 2.0 + size.y / 2.0 + 8.0;
          let bar_length = 224.0;
          let bar_height = 6.0;
          let value = (bar_length / 2.0 * (time.to_seconds() / break_length.to_seconds())) as f32;
          ui.painter().rect(
            egui::Rect::from_two_pos(egui::pos2(x - value, y), egui::pos2(x + value, y + bar_height)),
            egui::Rounding::same(6.0),
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 255),
            egui::Stroke::NONE,
          );

          let text = ui.painter().layout_no_wrap(
            String::from("Skip"),
            egui::FontId::proportional(SKIP_TEXT_SIZE),
            egui::Color32::PLACEHOLDER,
          );

          let y = y + size.y / 2.0;
          let size = text.size();
          let x = ui.available_width() / 2.0 - size.x / 2.0;
          ui.painter().galley(egui::pos2(x, y), text, ui.style().visuals.strong_text_color());
        });
      }
    });
  }
}
