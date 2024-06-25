use std::path::PathBuf;

use log::error;
use rodio::source::Empty;
use tap::Tap;
use winit::{
  event::{KeyEvent, Modifiers},
  keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

use crate::core::{
  app::App,
  audio::{
    self,
    audio_engine::AudioEngine,
    audio_mixer::{mixer, AudioController, AudioMixer},
  },
  core::Core,
  event::EventBus,
  graphics::{drawable::Drawable, graphics::Graphics},
  input::{
    bind::{Bind, KeyCombination},
    Input,
  },
};

use super::{
  event::ClientEvent,
  gameplay::{beatmap_cache::BeatmapCache, taiko_player::TaikoPlayerInput},
  input::client_action::ClientAction,
  screen::{
    gameplay_screen::gameplay_screen::GameplayScreen, result_screen::result_screen::ResultScreen,
    selection_screen::selection_screen::SelectionScreen, settings_screen::settings_screen::SettingsScreen,
  },
  state::AppState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalState {
  Selection,
  Playing,
  Results,
}

pub struct Client {
  input: Input<ClientAction>,
  audio_engine: AudioEngine,
  audio_controller: AudioController,
  event_bus: EventBus<ClientEvent>,

  game_state: LogicalState,

  beatmap_cache: BeatmapCache,

  /// Configuration and state of the whole game
  pub app_state: AppState,

  selection_screen: SelectionScreen,
  result_screen: ResultScreen,
  gameplay_screen: GameplayScreen,
  settings_screen: SettingsScreen,
}

impl App for Client {
  type Event = ClientEvent;

  fn prepare(&mut self, core: &mut Core<Self>, encoder: &mut wgpu::CommandEncoder) {
    core.egui_ctx.begin_frame(core.window);

    match self.game_state {
      LogicalState::Selection => {
        self.selection_screen.prepare(core, &self.beatmap_cache);
      }

      LogicalState::Playing => {
        self.gameplay_screen.prepare(core, &mut self.audio_engine, &self.app_state);
      }

      LogicalState::Results => {
        self.result_screen.prepare(core, &mut self.app_state, &self.beatmap_cache);
      }
    }

    self.settings_screen.prepare(core, &mut self.input, &mut self.app_state);

    core.egui_ctx.end_frame(&core.graphics, encoder);
  }

  fn render<'rpass>(&'rpass self, core: &'rpass mut Core<Self>, rpass: &mut wgpu::RenderPass<'rpass>) {
    // Draw wgpu
    match self.game_state {
      LogicalState::Selection => {}

      LogicalState::Playing => {
        self.gameplay_screen.render(rpass);
      }

      LogicalState::Results => {}
    }

    // Draw egui
    core.egui_ctx.render(&core.graphics, rpass);
  }

  fn resize(&mut self, core: &mut Core<Self>, size: winit::dpi::PhysicalSize<u32>) {
    self.gameplay_screen.resize(size);
  }

  fn scale(&mut self, core: &mut Core<Self>, scale_factor: f64) {
    self.gameplay_screen.scale(scale_factor);
  }
}

impl Drawable for Client {
  fn recreate(&mut self, graphics: &Graphics) {
    self.gameplay_screen.recreate(graphics);
  }
}

impl Client {
  pub fn new(core: &mut Core<Self>, app_state: AppState, event_bus: EventBus<ClientEvent>) -> Self {
    let input = Client::default_input();
    let (audio_mixer, audio_controller) = audio::mixer(Empty::new());
    let mut audio_engine = AudioEngine::new();
    audio_engine.set_source(audio_mixer);

    let game_state = LogicalState::Selection;

    let beatmap_cache = BeatmapCache::new().tap_mut(|cache| {
      cache.load_beatmaps("./beatmaps");
    });

    let selection_screen = SelectionScreen::new(event_bus.clone(), &beatmap_cache);
    let result_screen = ResultScreen::new(event_bus.clone(), &beatmap_cache, &PathBuf::new());
    #[rustfmt::skip]
    let gameplay_screen = GameplayScreen::new(event_bus.clone(), &core.graphics, &audio_engine, audio_controller.clone());
    let settings_screen = SettingsScreen::new(event_bus.clone());

    return Self {
      input,
      audio_engine,
      audio_controller,
      event_bus,
      game_state,
      app_state,
      beatmap_cache,
      selection_screen,
      gameplay_screen,
      settings_screen,
      result_screen,
    };
  }

  pub fn input(&mut self, core: &mut Core<Self>, event: KeyEvent) {
    if { true }
      && event.physical_key != PhysicalKey::Code(KeyCode::SuperRight)
      && event.physical_key != PhysicalKey::Code(KeyCode::SuperLeft)
      && event.physical_key != PhysicalKey::Code(KeyCode::ShiftLeft)
      && event.physical_key != PhysicalKey::Code(KeyCode::ShiftRight)
      && event.physical_key != PhysicalKey::Code(KeyCode::AltLeft)
      && event.physical_key != PhysicalKey::Code(KeyCode::AltRight)
      && event.physical_key != PhysicalKey::Code(KeyCode::ControlLeft)
      && event.physical_key != PhysicalKey::Code(KeyCode::ControlRight)
    {
      self.input.state.last_pressed = event.physical_key;

      if self.input.grabbing {
        self.input.grabbing = false;
        return;
      }
    }

    if event.state.is_pressed() {
      if self.game_state == LogicalState::Selection {
        match event.physical_key {
          PhysicalKey::Code(KeyCode::Backspace) => {
            if self.selection_screen.beatmap_selector().has_query() {
              self.selection_screen.beatmap_selector_mut().pop_query();
              return;
            }
          }

          PhysicalKey::Code(KeyCode::Escape) | PhysicalKey::Code(KeyCode::Enter) => {}
          PhysicalKey::Code(KeyCode::Comma) if self.input.state.modifiers.contains(ModifiersState::SUPER) => {}

          _ => {
            if let Some(c) = event.logical_key.to_text().and_then(|x| x.chars().next()) {
              self.selection_screen.beatmap_selector_mut().push_query(c);
            }
          }
        }
      }

      let comb = KeyCombination::new(event.physical_key, self.input.state.modifiers);
      if let Some(action) = self.input.keybinds.get(&comb).map(|x| x.id) {
        self.action(core, action, event.repeat);
      }
    }
  }

  pub fn modifiers(&mut self, modifiers: Modifiers) {
    self.input.state.modifiers = modifiers.state();
  }

  pub fn action(&mut self, core: &mut Core<Self>, action: ClientAction, repeat: bool) {
    match action {
      ClientAction::Back => {
        match self.game_state {
          LogicalState::Selection => {
            if self.settings_screen.is_settings_open() {
              self.settings_screen.toggle_settings();
            } else if self.selection_screen.beatmap_selector().has_query() {
              self.selection_screen.beatmap_selector_mut().clear_query();
            } else {
              core.exit();
            }
          }

          LogicalState::Playing => {
            if self.settings_screen.is_settings_open() {
              self.settings_screen.toggle_settings();
            } else {
              self.game_state = LogicalState::Selection;
            }
          }

          LogicalState::Results => {
            self.game_state = LogicalState::Selection;
          }
        }
      }

      ClientAction::Settings => {
        self.settings_screen.toggle_settings();
      }

      ClientAction::Next => {
        match self.game_state {
          LogicalState::Selection => {
            self.selection_screen.beatmap_selector_mut().select_next();
          }

          _ => {}
        }
      }

      ClientAction::Prev => {
        match self.game_state {
          LogicalState::Selection => {
            self.selection_screen.beatmap_selector_mut().select_prev();
          }

          _ => {}
        }
      }

      ClientAction::Retry => {
        self.event_bus.send(ClientEvent::RetryBeatmap);
      }

      ClientAction::Select => {
        match self.game_state {
          LogicalState::Selection => {
            self
              .selection_screen
              .beatmap_selector()
              .select(&self.event_bus, &self.beatmap_cache)
              .unwrap_or_else(|err| {
                error!("Failed to select beatmap: {:?}", err);
              });
          }

          _ => {}
        }
      }

      ClientAction::KatOne if !repeat => {
        self
          .gameplay_screen
          .hit(TaikoPlayerInput::KatOne, &core.graphics, &mut self.audio_engine, &self.app_state);
      }

      ClientAction::KatTwo if !repeat => {
        self
          .gameplay_screen
          .hit(TaikoPlayerInput::KatTwo, &core.graphics, &mut self.audio_engine, &self.app_state);
      }

      ClientAction::DonOne if !repeat => {
        self
          .gameplay_screen
          .hit(TaikoPlayerInput::DonOne, &core.graphics, &mut self.audio_engine, &self.app_state);
      }

      ClientAction::DonTwo if !repeat => {
        self
          .gameplay_screen
          .hit(TaikoPlayerInput::DonTwo, &core.graphics, &mut self.audio_engine, &self.app_state);
      }

      _ => {}
    }
  }

  pub fn dispatch(&mut self, core: &mut Core<Self>, event: ClientEvent) {
    match event {
      ClientEvent::SelectBeatmap { path } => {
        self.game_state = LogicalState::Playing;
        self.gameplay_screen.play(&path, &core.graphics, &mut self.audio_engine, &self.app_state);
      }

      ClientEvent::RetryBeatmap => {
        self.gameplay_screen.reset(&core.graphics, &mut self.audio_engine);
      }

      ClientEvent::ToggleSettings => {
        self.settings_screen.toggle_settings();
      }

      ClientEvent::RebuildTaikoRendererInstances => {
        self.gameplay_screen.rebuild_instances(&core.graphics, &self.app_state);
      }

      ClientEvent::ShowResultScreen { path } => {
        self.game_state = LogicalState::Results;
        self.result_screen.finish(&self.beatmap_cache, &path);
      }
    }
  }

  pub fn file(&mut self, core: &mut Core<Self>, path: PathBuf, file: Vec<u8>) {
    // TODO: this logic should be moved to the beatmap manager or whatever
    // TODO: properly parse beatmapset id
    let beatmapset_id = path.file_name().unwrap().to_str().unwrap().split_whitespace().next().unwrap();
    zip::read::ZipArchive::new(std::io::Cursor::new(file))
      .unwrap()
      .extract(&format!("./beatmaps/{}", beatmapset_id))
      .unwrap();

    self.beatmap_cache.load_difficulties(&format!("./beatmaps/{}", beatmapset_id));
  }

  fn default_input() -> Input<ClientAction> {
    let mut input = Input::default();

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Comma), ModifiersState::SUPER),
      Bind {
        id: ClientAction::Settings,
        name: String::from("Settings"),
        description: String::from("Open settings menu"),
      },
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Escape), ModifiersState::empty()),
      Bind {
        id: ClientAction::Back,
        name: String::from("Back"),
        description: String::from("Return to the previous state"),
      },
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Enter), ModifiersState::empty()),
      Bind {
        id: ClientAction::Select,
        name: String::from("Select"),
        description: String::from("Pick selected element"),
      },
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Backquote), ModifiersState::empty()),
      Bind {
        id: ClientAction::Retry,
        name: String::from("Retry"),
        description: String::from("Replay a beatmap from the beginning"),
      },
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::ArrowDown), ModifiersState::empty()),
      Bind {
        id: ClientAction::Next,
        name: String::from("Next"),
        description: String::from("Select next element"),
      },
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::ArrowUp), ModifiersState::empty()),
      Bind {
        id: ClientAction::Prev,
        name: String::from("Previous"),
        description: String::from("Select previous element"),
      },
    );

    // Gameplay control
    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::KeyS), ModifiersState::empty()),
      Bind {
        id: ClientAction::KatOne,
        name: String::from("Kat 1"),
        description: String::from("Kat (blue)"),
      },
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::KeyL), ModifiersState::empty()),
      Bind {
        id: ClientAction::KatTwo,
        name: String::from("Kat 2"),
        description: String::from("Kat (blue)"),
      },
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::KeyD), ModifiersState::empty()),
      Bind {
        id: ClientAction::DonOne,
        name: String::from("Don 1"),
        description: String::from("Don (red)"),
      },
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::KeyK), ModifiersState::empty()),
      Bind {
        id: ClientAction::DonTwo,
        name: String::from("Don 2"),
        description: String::from("Don (red)"),
      },
    );

    return input;
  }
}
