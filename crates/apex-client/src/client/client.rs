use log::error;
use tap::Tap;
use winit::{event::{KeyEvent, Modifiers}, keyboard::{KeyCode, ModifiersState, PhysicalKey}};

use crate::core::{app::App, core::Core, event::EventBus, input::{bind::{Bind, KeyCombination}, Input}};

use super::{event::ClientEvent, gameplay::{beatmap_cache::BeatmapCache, taiko_player::TaikoPlayerInput}, input::client_action::ClientAction, screen::{gameplay_screen::gameplay_screen::GameplayScreen, selection_screen::selection_screen::SelectionScreen}, state::GameState};

pub struct Client {
  input     : Input<ClientAction>,
  event_bus : EventBus<ClientEvent>,

  game_state : GameState,

  beatmap_cache : BeatmapCache,

  selection_screen : SelectionScreen,
  gameplay_screen  : GameplayScreen,
}

impl App for Client {
  type Event = ClientEvent;

  fn prepare(&mut self, core: &mut Core<Self>, encoder: &mut wgpu::CommandEncoder) {
    core.egui_ctx.begin_frame(core.window);

    match self.game_state {
      GameState::Selection => {
        self.selection_screen.prepare(core, &self.beatmap_cache);
      }

      GameState::Playing => {
        self.gameplay_screen.prepare(core);
      }
    }

    core.egui_ctx.end_frame(&core.graphics, encoder);
  }

  fn render<'rpass>(&'rpass self, core: &'rpass mut Core<Self>, rpass: &mut wgpu::RenderPass<'rpass>) {
    // Draw wgpu
    match self.game_state {
      GameState::Selection => {
      }

      GameState::Playing => {
        self.gameplay_screen.render(rpass);
      }
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

impl Client {
  pub fn new(core: &mut Core<Self>, event_bus: EventBus<ClientEvent>) -> Self {
    let mut input = Input::default();

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Escape), ModifiersState::empty()),
      Bind {
        id: ClientAction::Back,
        name: String::from("Back"),
        description: String::from("Return to the previous state"),
      }
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Enter), ModifiersState::empty()),
      Bind {
        id: ClientAction::Select,
        name: String::from("Select"),
        description: String::from("Pick selected element"),
      }
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Backquote), ModifiersState::empty()),
      Bind {
        id: ClientAction::Retry,
        name: String::from("Retry"),
        description: String::from("Replay a beatmap from the beginning"),
      }
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::ArrowDown), ModifiersState::empty()),
      Bind {
        id: ClientAction::Next,
        name: String::from("Next"),
        description: String::from("Select next element"),
      }
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::ArrowUp), ModifiersState::empty()),
      Bind {
        id: ClientAction::Prev,
        name: String::from("Previous"),
        description: String::from("Select previous element"),
      }
    );

    // Gameplay control
    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::KeyA), ModifiersState::empty()),
      Bind {
        id: ClientAction::KatOne,
        name: String::from("Kat 1"),
        description: String::from("Kat (blue)"),
      }
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Quote), ModifiersState::empty()),
      Bind {
        id: ClientAction::KatTwo,
        name: String::from("Kat 2"),
        description: String::from("Kat (blue)"),
      }
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::KeyS), ModifiersState::empty()),
      Bind {
        id: ClientAction::DonOne,
        name: String::from("Don 1"),
        description: String::from("Don (red)"),
      }
    );

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Semicolon), ModifiersState::empty()),
      Bind {
        id: ClientAction::DonTwo,
        name: String::from("Don 2"),
        description: String::from("Don (red)"),
      }
    );

    let game_state = GameState::Selection;
    let beatmap_cache = BeatmapCache::new().tap_mut(|cache| {
      cache.load_beatmaps("/Users/polina4096/dev/apex/test/beatmaps");
    });

    let selection_screen = SelectionScreen::new(event_bus.clone(), &beatmap_cache);
    let gameplay_screen = GameplayScreen::new(&core.graphics);

    return Self {
      input,
      event_bus,
      game_state,
      beatmap_cache,
      selection_screen,
      gameplay_screen,
    };
  }

  pub fn input(&mut self, core: &mut Core<Self>, event: KeyEvent) {
    if event.state.is_pressed() {
      if self.game_state == GameState::Selection {
        match event.physical_key {
          PhysicalKey::Code(KeyCode::Backspace) => {
            if self.selection_screen.beatmap_selector().has_query() {
              self.selection_screen.beatmap_selector_mut().pop_query();
              return;
            }
          }

          | PhysicalKey::Code(KeyCode::Escape)
          | PhysicalKey::Code(KeyCode::Enter)
          => { }

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
          GameState::Selection => {
            if self.selection_screen.beatmap_selector().has_query() {
              self.selection_screen.beatmap_selector_mut().clear_query();
            } else {
              core.exit();
            }
          }

          GameState::Playing => {
            self.game_state = GameState::Selection;
          }
        }
      }

      ClientAction::Next => {
        match self.game_state {
          GameState::Selection => {
            self.selection_screen.beatmap_selector_mut().select_next();
          }

          _ => { }
        }
      }

      ClientAction::Prev => {
        match self.game_state {
          GameState::Selection => {
            self.selection_screen.beatmap_selector_mut().select_prev();
          }

          _ => { }
        }
      }

      ClientAction::Retry => {
        self.event_bus.send(ClientEvent::RetryBeatmap);
      }

      ClientAction::Select => {
        match self.game_state {
          GameState::Selection => {
            self.selection_screen.beatmap_selector().select(&self.event_bus, &self.beatmap_cache)
              .unwrap_or_else(|err| { error!("Failed to select beatmap: {:?}", err); });
          }

          _ => { }
        }
      }

      ClientAction::KatOne if !repeat => {
        self.gameplay_screen.hit(&core.graphics, TaikoPlayerInput::KatOne);
      }
      ClientAction::KatTwo if !repeat => {
        self.gameplay_screen.hit(&core.graphics, TaikoPlayerInput::KatTwo);
      }
      ClientAction::DonOne if !repeat => {
        self.gameplay_screen.hit(&core.graphics, TaikoPlayerInput::DonOne);
      }
      ClientAction::DonTwo if !repeat => {
        self.gameplay_screen.hit(&core.graphics, TaikoPlayerInput::DonTwo);
      }

      _ => { }
    }
  }

  pub fn dispatch(&mut self, core: &mut Core<Self>, event: ClientEvent) {
    match event {
      ClientEvent::SelectBeatmap { path } => {
        self.game_state = GameState::Playing;
        self.gameplay_screen.play(&path, &core.graphics);
      }

      ClientEvent::RetryBeatmap => {
        self.gameplay_screen.reset(&core.graphics);
      }
    }
  }
}
