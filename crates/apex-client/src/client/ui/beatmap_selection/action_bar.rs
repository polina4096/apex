use egui::Widget as _;
use tap::Tap;

use crate::{
  client::event::ClientEvent,
  core::{
    event::EventBus,
    time::{clock::AbstractClock, time::Time},
  },
};

pub struct ActionBar {
  event_bus: EventBus<ClientEvent>,
}

impl ActionBar {
  pub fn new(event_bus: EventBus<ClientEvent>) -> Self {
    return Self { event_bus };
  }

  pub fn prepare(&self, ui: &mut egui::Ui, clock: &mut impl AbstractClock) {
    egui::Frame::window(ui.style())
      .outer_margin(egui::Margin {
        left: 12.0,
        right: 0.0,
        top: 12.0,
        bottom: 12.0,
      })
      .inner_margin(egui::Margin::symmetric(20.0, 16.0))
      .show(ui, |ui| {
        let text = egui::RichText::new("⛭").line_height(Some(24.0)).size(24.0);
        if egui::Button::new(text).frame(false).ui(ui).clicked() {
          self.event_bus.send(ClientEvent::ToggleSettings);
        }
      });

    egui::Frame::window(ui.style())
      .outer_margin(egui::Margin {
        left: 2.0,
        right: 0.0,
        top: 12.0,
        bottom: 12.0,
      })
      .inner_margin(egui::Margin::symmetric(20.0, 16.0))
      .show(ui, |ui| {
        let symbol = if clock.is_playing() { "⏸" } else { "⏵" };
        let text = egui::RichText::new(symbol).line_height(Some(24.0)).size(24.0);
        if egui::Button::new(text).frame(false).ui(ui).clicked() {
          clock.toggle();
        }

        egui::Frame::none() //
          .inner_margin(egui::Margin {
            left: 6.0,
            right: 0.0,
            top: 0.0,
            bottom: 2.5,
          })
          .show(ui, |ui| {
            let mut pos = clock.position().to_seconds();
            ui.style_mut().spacing.slider_width = 128.0;
            if egui::Slider::new(&mut pos, 0.0 ..= clock.length().to_seconds())
              .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.5 })
              .show_value(false)
              .smart_aim(false)
              .trailing_fill(true)
              .ui(ui)
              .changed()
            {
              clock.set_position(Time::from_seconds(pos));
            }

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
              let text = format!("{:.2}s", pos);
              egui::Label::new(egui::RichText::new(text).size(16.0).line_height(Some(18.0))).ui(ui);
            });
          })
      });
  }
}