use std::path::Path;

use egui::Widget;

use crate::{
  client::{event::ClientEvent, gameplay::beatmap_cache::BeatmapInfo, ui::card_component::CardComponent},
  core::event::EventBus,
};

pub struct BeatmapCard {
  card: CardComponent,
  title: egui::RichText,
}

impl BeatmapCard {
  pub fn new(path: &Path, info: &BeatmapInfo) -> Self {
    let bg = path.parent().unwrap().join(&info.bg_path);
    let bg = format!("file://{}", bg.to_str().unwrap());

    let card = CardComponent::new(bg);
    let title = egui::RichText::new(format!("{} {} {}", info.artist, info.title, info.variant)).strong();

    return Self { card, title };
  }

  pub fn prepare(
    &mut self,
    ui: &mut egui::Ui,
    selected: bool,
    path: &Path,
    bus: &EventBus<ClientEvent>,
  ) -> egui::Response {
    self.card.selected = selected;

    let card = self.card.prepare(ui, 64.0, |ui| {
      egui::Frame::none() //
        .inner_margin(egui::Margin::same(8.0))
        .show(ui, |ui| {
          egui::Label::new(self.title.clone()).ui(ui);
        });
    });

    if card.hovered() {
      ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    card.context_menu(|ui| {
      ui.set_max_width(128.0);
      ui.style_mut().spacing.button_padding = egui::vec2(6.0, 2.0);

      const FONT_SIZE: f32 = 13.0;

      let text = egui::RichText::new("Play").size(FONT_SIZE);
      if ui.button(text).clicked() {
        let path = path.to_owned();
        bus.send(ClientEvent::PickBeatmap { path });

        ui.close_menu();
      }

      let text = egui::RichText::new("Record").size(FONT_SIZE);
      if ui.button(text).clicked() {
        bus.send(ClientEvent::ToggleRecordingWindow);

        ui.close_menu();
      }

      ui.add_enabled_ui(false, |ui| {
        let text = egui::RichText::new("Export").size(FONT_SIZE);
        if ui.button(text).clicked() {
          ui.close_menu();
        }
      });

      ui.separator();

      ui.add_enabled_ui(false, |ui| {
        let text = egui::RichText::new("Edit").size(FONT_SIZE);
        if ui.button(text).clicked() {
          ui.close_menu();
        }

        let text = egui::RichText::new("Copy").size(FONT_SIZE);
        if ui.button(text).clicked() {
          ui.close_menu();
        }

        let text = egui::RichText::new("Delete").size(FONT_SIZE);
        if ui.button(text).clicked() {
          ui.close_menu();
        }
      });
    });

    return card;
  }
}
