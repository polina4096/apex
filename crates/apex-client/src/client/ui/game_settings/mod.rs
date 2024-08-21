use egui::Widget;
use log::debug;

use apex_framework::input::{
  keybinds::{Bind, KeyCombination},
  Input,
};
use tap::Tap;

use crate::client::{
  action::ClientAction,
  settings::{Settings, SettingsKind, SettingsProxy},
};

pub mod tab_controls;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSettingsTab {
  General,
  Controls,
}

pub struct GameSettingsView {
  pub tab: GameSettingsTab,
  pub is_open: bool,

  buffer: String,
  current_bind: Option<KeyCombination>,
  bind_cache: Vec<(KeyCombination, Bind<ClientAction>)>,
}

impl GameSettingsView {
  pub fn new() -> Self {
    return Self {
      tab: GameSettingsTab::General,
      is_open: false,

      buffer: String::new(),
      current_bind: None,
      bind_cache: vec![],
    };
  }

  pub fn prepare(
    &mut self,
    ctx: &egui::Context,
    input: &mut Input<ClientAction>,
    settings: &mut Settings,
    proxy: &mut impl SettingsProxy,
  ) {
    // TODO: the cache won't be rebuilt if the keybinds are changed while the,
    // settings are open yet it doesn't matter right now as that is not possible.
    if !self.is_open {
      self.bind_cache.clear();
    } else if self.bind_cache.is_empty() && !input.keybinds.is_empty() {
      debug!("Rebuilding bind cache");
      self.bind_cache = input.keybinds.as_vec();
    }

    let width = 512.0;
    let offset = ctx.animate_value_with_time(
      egui::Id::new("settings_expand_anim"),
      if !self.is_open { width } else { 0.0 },
      0.125,
    );

    if width <= 0.0 {
      return;
    }

    egui::Window::new("settings")
      .movable(false)
      .resizable(false)
      .title_bar(false)
      .fixed_size([width, ctx.screen_rect().height()])
      .fixed_pos([0.0, 0.0])
      .frame(
        egui::Frame::none()
          .fill(egui::Color32::from_rgba_unmultiplied(4, 4, 4, 253))
          .outer_margin(egui::Margin { left: -offset, ..Default::default() }),
      )
      .show(ctx, |ui| {
        egui_extras::StripBuilder::new(ui)
          .size(egui_extras::Size::exact(48.0))
          .size(egui_extras::Size::remainder())
          .horizontal(|mut strip| {
            let mut scroll = None;

            strip.cell(|ui| {
              let padding = 8.0;

              egui::Frame::none() //
                .fill(egui::Color32::from_gray(20))
                .inner_margin(egui::Margin::same(padding))
                .show(ui, |ui| {
                  let button_count = settings.group_count();
                  let button_size = 32.0;

                  let buttons_panel = button_size * button_count as f32;
                  let offset = ui.available_height() / 2.0 - buttons_panel / 2.0 - button_size - padding;

                  ui.vertical_centered(|ui| {
                    if egui::Button::new("⛭")
                      .frame(false)
                      .min_size(egui::vec2(button_size, button_size))
                      .ui(ui)
                      .on_hover_cursor(egui::CursorIcon::PointingHand)
                      .clicked()
                    {
                      self.is_open = false;
                    }

                    ui.add_space(offset);

                    settings.ui_sidebar(ui, &mut scroll);

                    ui.add_space(offset - padding + 3.0);

                    if egui::Button::new("⏴")
                      .frame(false)
                      .min_size(egui::vec2(button_size, button_size))
                      .ui(ui)
                      .on_hover_cursor(egui::CursorIcon::PointingHand)
                      .clicked()
                    {
                      self.is_open = false;
                    }
                  });
                });
            });

            strip.cell(|ui| {
              egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                  egui::Frame::none() //
                    .inner_margin(egui::Margin::same(20.0).tap_mut(|x| x.right = 4.0))
                    .show(ui, |ui| {
                      ui.label(egui::RichText::new("Settings").strong().size(32.0));

                      ui.add_space(16.0);

                      settings.ui(ui, proxy, scroll);
                      // self.controls_tab(ui, input);
                    });
                });
            });
          });

        ui.painter().line_segment(
          [
            egui::pos2(width - 1.0 - offset, 0.0),
            egui::pos2(width - 1.0 - offset, ui.ctx().screen_rect().height()),
          ],
          egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(128, 128, 128, 24)),
        );

        ui.allocate_space(ui.available_size());
      });
  }
}
