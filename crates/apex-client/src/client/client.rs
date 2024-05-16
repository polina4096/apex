use tap::Tap;
use winit::{event::{KeyEvent, Modifiers}, keyboard::{KeyCode, ModifiersState, PhysicalKey}};

use crate::core::{app::App, core::Core, event::EventBus, input::{bind::{Bind, KeyCombination}, Input}};

use super::{event::ClientEvent, input::client_action::ClientAction, screen::{gameplay_screen::gameplay_screen::GameplayScreen, selection_screen::selection_screen::SelectionScreen}, state::GameState, taiko::beatmap_cache::BeatmapCache};

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
        self.selection_screen.prepare(core);
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

    input.keybinds.add(
      KeyCombination::new(PhysicalKey::Code(KeyCode::Enter), ModifiersState::empty()),
      Bind {
        id: ClientAction::Select,
        name: String::from("Select"),
        description: String::from("Pick selected element"),
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
      let comb = KeyCombination::new(event.physical_key, self.input.state.modifiers);
      if let Some(action) = self.input.keybinds.get(&comb).map(|x| x.id) {
        self.action(core, action)
      }
    }
  }

  pub fn modifiers(&mut self, modifiers: Modifiers) {
    self.input.state.modifiers = modifiers.state();
  }

  pub fn action(&mut self, core: &mut Core<Self>, action: ClientAction) {
    match action {
      ClientAction::Back => {
        match self.game_state {
          GameState::Selection => {
            core.exit();
          }

          GameState::Playing => {
            self.game_state = GameState::Selection;
          }
        }
      }

      ClientAction::Next => {
        match self.game_state {
          GameState::Selection => {
            self.selection_screen.select_next();
          }

          _ => { }
        }
      }

      ClientAction::Prev => {
        match self.game_state {
          GameState::Selection => {
            self.selection_screen.select_prev();
          }

          _ => { }
        }
      }

      ClientAction::Select => {
        match self.game_state {
          GameState::Selection => {
            self.selection_screen.select();
          }

          _ => { }
        }
      }
    }
  }

  pub fn dispatch(&mut self, core: &mut Core<Self>, event: ClientEvent) {
    match event {
      ClientEvent::SelectBeatmap { path } => {
        self.game_state = GameState::Playing;
        self.gameplay_screen.play(&path, &core.graphics);
      }
    }
  }
}
