use egui::ImageSource;

use crate::{
  client::{client::Client, state::AppState},
  core::core::Core,
};

use super::{background_component::BackgroundComponent, card_component::CardComponent};

pub struct PlayResultsView {
  preview: CardComponent,
  background: BackgroundComponent,
}

impl PlayResultsView {
  pub fn new(source: impl Into<ImageSource<'static>>) -> Self {
    let image = source.into();

    let background = BackgroundComponent::new(image.clone());
    let preview = CardComponent::new(image.clone());

    return Self { preview, background };
  }

  pub fn prepare(&mut self, core: &Core<Client>, state: &mut AppState) {
    let frame = egui::Frame::none() //
      .inner_margin(egui::Margin::ZERO);

    egui::CentralPanel::default().frame(frame).show(core.egui_ctx(), |ui| {
      self.background.prepare(ui);

      ui.set_height(ui.available_height());
      ui.set_width(ui.available_width());
      self.preview.prepare(ui, |ui| {
        ui.label("gay");
      });
    });
  }
}
