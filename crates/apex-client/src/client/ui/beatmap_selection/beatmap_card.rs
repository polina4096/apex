use apex_framework::event::EventBus;
use egui::Widget;

use crate::client::{
  event::ClientEvent,
  gameplay::{beatmap::BeatmapHash, beatmap_cache::BeatmapInfo},
  ui::card_component::CardComponent,
};

pub struct BeatmapCard {
  card: CardComponent,
  artist: String,
  title: String,
  variant: String,
  difficulty: f64,
}

impl BeatmapCard {
  pub fn new(info: &BeatmapInfo) -> Self {
    let bg = info.file_path.parent().unwrap().join(&info.bg_path);
    let bg = format!("file://{}", bg.to_str().unwrap());

    return Self {
      card: CardComponent::new(bg),
      artist: info.artist.clone(),
      title: info.title.clone(),
      variant: info.variant.clone(),
      difficulty: info.difficulty,
    };
  }

  pub fn prepare(
    &mut self,
    ui: &mut egui::Ui,
    selected: bool,
    beatmap_hash: BeatmapHash,
    bus: &EventBus<ClientEvent>,
  ) -> egui::Response {
    self.card.selected = selected;

    let card = self.card.prepare(ui, 64.0, |ui| {
      egui::Frame::none() //
        .inner_margin(egui::Margin::same(8.0))
        .show(ui, |ui| {
          ui.horizontal(|ui| {
            ui.vertical(|ui| {
              egui::Label::new(egui::RichText::new(&self.title).strong().size(14.0)).truncate().ui(ui);

              ui.add_space(2.0);

              egui::Label::new(egui::RichText::new(&self.artist).strong().size(12.0)).truncate().ui(ui);

              ui.add_space(10.0);
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
              ui.add_space(4.0);

              ui.label(
                egui::RichText::new(format!("{}  ∙  {:.2} ★", &self.variant, self.difficulty)).strong().size(12.0),
              );
            });
          });
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
        bus.send(ClientEvent::PickBeatmap { beatmap_hash });

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
