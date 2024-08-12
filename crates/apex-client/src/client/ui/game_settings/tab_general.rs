use crate::client::settings::{Settings, SettingsProxy};

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
      .column(Column::exact(160.0))
      .column(Column::remainder())
      .body(|mut body| {
        settings.ui(&mut body, text_height, proxy);
      });
  }
}
