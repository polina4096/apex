use crate::{
  client::{
    client::{Client, GameState},
    event::ClientEvent,
    gameplay::beatmap_cache::BeatmapCache,
    screen::{gameplay_screen::gameplay_screen::GameplayScreen, selection_screen::selection_screen::SelectionScreen},
    settings::Settings,
  },
  core::{
    audio::{audio_engine::AudioEngine, audio_mixer::AudioController},
    core::Core,
    event::EventBus,
    time::{clock::AbstractClock as _, time::Time},
  },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedButton {
  None,
  Continue,
  Retry,
  Quit,
}

pub struct PauseScreen {
  event_bus: EventBus<ClientEvent>,

  clicked: bool,
  selected_button: SelectedButton,
}

impl PauseScreen {
  pub fn new(event_bus: EventBus<ClientEvent>) -> Self {
    let clicked = false;
    let selected_button = SelectedButton::None;
    return Self { event_bus, clicked, selected_button };
  }

  #[allow(clippy::too_many_arguments)]
  pub fn prepare(
    &mut self,
    core: &Core<Client>,
    audio_engine: &mut AudioEngine,
    audio_controller: &mut AudioController,
    gameplay_screen: &mut GameplayScreen,
    selection_screen: &mut SelectionScreen,
    beatmap_cache: &BeatmapCache,
    game_state: &mut GameState,
    settings: &Settings,
  ) {
    egui::CentralPanel::default() //
      .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220)))
      .show(core.egui_ctx.egui_ctx(), |ui| {
        const TEXT_SIZE: f32 = 32.0;

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
          fn draw_button(
            ui: &egui::Ui,
            text: &str,
            additional_y: f32,
            selected: &mut SelectedButton,
            current: SelectedButton,
            clicked: &mut bool,
            click: impl FnOnce(),
          ) {
            let max_width = ui.available_width();
            let max_height = ui.available_height();

            let inactive_color = egui::Color32::from_rgba_unmultiplied(50, 50, 50, 50);
            let active_color = egui::Color32::from_rgba_unmultiplied(100, 100, 150, 50);

            let id = egui::Id::new(text);
            let text = ui.painter().layout_no_wrap(
              text.to_owned(),
              egui::FontId::proportional(TEXT_SIZE),
              egui::Color32::PLACEHOLDER,
            );

            let item_height = text.size().y;
            let x = max_width / 2.0 - text.size().x / 2.0;
            let y = max_height / 2.0 + item_height * additional_y - item_height / 2.0;

            let rect = egui::Rect::from_two_pos(egui::pos2(0.0, y), egui::pos2(max_width, y + item_height));
            let response = ui.interact(rect, id, egui::Sense::click());

            let mut fg_color = ui.style().visuals.text_color();
            let mut bg_color = inactive_color;

            if response.hovered() && ui.input(|x| x.pointer.is_moving()) {
              *selected = current;
            }

            let is_selected = *selected == current;

            if is_selected {
              fg_color = ui.style().visuals.strong_text_color();
              bg_color = active_color;
            }

            let is_clicked = *clicked && is_selected;

            if response.clicked() || is_clicked {
              if is_clicked {
                *clicked = false;
                *selected = SelectedButton::None;
              }

              click();
            }

            ui.painter().rect(rect, egui::Rounding::ZERO, bg_color, egui::Stroke::NONE);
            ui.painter().galley(egui::pos2(x, y), text, fg_color);
          }

          let selected = &mut self.selected_button;
          draw_button(ui, "Continue", -1.0, selected, SelectedButton::Continue, &mut self.clicked, || {
            *game_state = GameState::Playing;

            gameplay_screen.set_paused(false, audio_engine);
          });

          draw_button(ui, "Retry", 0.0, selected, SelectedButton::Retry, &mut self.clicked, || {
            *game_state = GameState::Playing;

            self.event_bus.send(ClientEvent::RetryBeatmap);
          });

          draw_button(ui, "Quit", 1.0, selected, SelectedButton::Quit, &mut self.clicked, || {
            *game_state = GameState::Selection;

            let lead_in = Time::from_ms(settings.gameplay.lead_in() as f64);
            let delay_adjusted_position = audio_engine.position() - lead_in;
            let delay_adjusted_position = delay_adjusted_position.max(Time::zero());

            let selected = selection_screen.beatmap_selector().selected();
            if let Some((path, beatmap)) = beatmap_cache.get_index(selected) {
              Client::play_beatmap_audio_unchecked(audio_engine, audio_controller, path, beatmap);
              audio_engine.set_position(delay_adjusted_position);
            };
          });
        });
      });
  }

  pub fn deselect(&mut self) {
    self.selected_button = SelectedButton::None;
  }

  pub fn select_up(&mut self) {
    self.selected_button = match self.selected_button {
      SelectedButton::None => SelectedButton::Quit,
      SelectedButton::Continue => SelectedButton::Quit,
      SelectedButton::Retry => SelectedButton::Continue,
      SelectedButton::Quit => SelectedButton::Retry,
    };
  }

  pub fn select_down(&mut self) {
    self.selected_button = match self.selected_button {
      SelectedButton::Continue => SelectedButton::Retry,
      SelectedButton::Retry => SelectedButton::Quit,
      SelectedButton::Quit => SelectedButton::Continue,
      SelectedButton::None => SelectedButton::Continue,
    };
  }

  pub fn click(&mut self) {
    self.clicked = true;
  }
}
