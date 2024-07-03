use std::{fs::File, io::BufReader, path::PathBuf};

use log::error;
use rodio::{
  source::{Empty, UniformSourceIterator},
  Decoder, DeviceTrait as _, Source,
};
use tap::Tap;
use winit::{
  event::{KeyEvent, Modifiers},
  keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

use crate::core::{
  app::App,
  audio::{self, audio_engine::AudioEngine, audio_mixer::AudioController},
  core::Core,
  event::EventBus,
  graphics::{color::Color, drawable::Drawable, graphics::Graphics},
  input::{
    bind::{Bind, KeyCombination},
    Input,
  },
  time::{clock::AbstractClock, time::Time},
};

use super::{
  event::ClientEvent,
  gameplay::{beatmap::Beatmap, beatmap_cache::BeatmapCache, taiko_player::TaikoPlayerInput},
  graphics::{
    taiko_renderer::taiko_renderer::{TaikoRenderer, TaikoRendererConfig},
    // video_exporter::VideoExporter,
  },
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

  prev_audio_path: PathBuf,
  prev_beatmap_path: PathBuf,

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
        self.selection_screen.prepare(core, &self.beatmap_cache, &mut self.audio_engine);
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
    self.gameplay_screen.resize(&core.graphics.queue, size);
  }

  fn scale(&mut self, core: &mut Core<Self>, scale_factor: f64) {
    self.gameplay_screen.scale(&core.graphics.queue, scale_factor);
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

    #[rustfmt::skip] let selection_screen = SelectionScreen::new(event_bus.clone(), &beatmap_cache, &mut audio_engine, &core.graphics, &mut core.egui_ctx, &app_state);
    #[rustfmt::skip] let result_screen = ResultScreen::new(event_bus.clone(), &beatmap_cache, &PathBuf::new());
    #[rustfmt::skip] let gameplay_screen = GameplayScreen::new(event_bus.clone(), &core.graphics, &audio_engine, audio_controller.clone(), &app_state);
    #[rustfmt::skip] let settings_screen = SettingsScreen::new();

    let prev_audio_path = PathBuf::new();
    let prev_beatmap_path = PathBuf::new();

    // render a play
    // {
    //   let mut video_exporter = VideoExporter::new(&core.graphics.device, core.graphics.format);

    //   let mut taiko_renderer = TaikoRenderer::new(
    //     &core.graphics,
    //     TaikoRendererConfig {
    //       width: 2048,
    //       height: 2048,
    //       scale_factor: 2.0,
    //       scale: 0.85,
    //       zoom: 0.235,
    //       hit_position_x: 128.0,
    //       hit_position_y: 256.0,
    //       don: Color::new(0.92, 0.00, 0.27, 1.00),
    //       kat: Color::new(0.00, 0.47, 0.67, 1.00),
    //     },
    //   );

    //   // let beatmap_path = beatmap_cache.get_index(0).unwrap().0;
    //   let data = std::fs::read_to_string("/Users/polina4096/dev/apex/debug/beatmaps/1796495/Envy - LIVING LIFE IN THE NIGHT (Genjuro) [SLEEP DEPRIVED].osu").unwrap();
    //   let beatmap = Beatmap::from(data);
    //   let info = beatmap_cache
    //     .get(&PathBuf::from("./beatmaps/1796495/Envy - LIVING LIFE IN THE NIGHT (Genjuro) [SLEEP DEPRIVED].osu"))
    //     .unwrap();

    //   taiko_renderer.load_beatmap(&core.graphics.device, beatmap);
    //   taiko_renderer.set_hit_all(&core.graphics.queue);

    //   video_exporter.export(&core.graphics, 0 .. (120 * 8), |rpass, graphics, i| {
    //     let time = Time::from_ms(i as f64 / 120.0 * 1000.0 + info.preview_time as f64);
    //     taiko_renderer.prepare(&graphics.queue, time);

    //     taiko_renderer.render(rpass);
    //   });

    //   // let (tx_data, rx_data) = std::sync::mpsc::sync_channel::<(i32, Vec<u8>)>(100);

    //   // std::thread::spawn(move || {
    //   //   let mut buffer = vec![0u8; 2048 * 2048 * 4];

    //   //   loop {
    //   //     let (i, data) = rx_data.recv().unwrap();
    //   //     for (i, chunk) in data.iter().copied().array_chunks::<16>().enumerate() {
    //   //       buffer[i * 12 .. (i + 1) * 12].copy_from_slice(&convert_swizzle(chunk));
    //   //     }

    //   //     pub fn convert_swizzle(bgra: [u8; 16]) -> [u8; 12] {
    //   //       use std::simd::{simd_swizzle, Simd};
    //   //       let bgra = Simd::from_array(bgra);
    //   //       #[rustfmt::skip]
    //   //       const IDXS: [usize; 16] = [
    //   //           2, 1, 0,
    //   //           6, 5, 4,
    //   //           10, 9, 8,
    //   //           14, 13, 12,
    //   //           3, 7, 11, 15,
    //   //       ];
    //   //       let rgb = simd_swizzle!(bgra, IDXS);
    //   //       return rgb.to_array().as_chunks().0[0];
    //   //     }

    //   //     use image::{ImageBuffer, Rgb};
    //   //     let buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(2048, 2048, buffer.clone()).unwrap();
    //   //     buffer.save(format!("/Users/polina4096/Desktop/apex/image{:08}.bmp", i)).unwrap();
    //   //   }
    //   // });
    // }

    return Self {
      input,
      audio_engine,
      audio_controller,
      event_bus,
      game_state,
      app_state,
      prev_audio_path,
      prev_beatmap_path,
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
            self.play_beatmap_audio();
          }

          _ => {}
        }
      }

      ClientAction::Prev => {
        match self.game_state {
          LogicalState::Selection => {
            self.selection_screen.beatmap_selector_mut().select_prev();
            self.play_beatmap_audio();
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
              .pick(&self.event_bus, &self.beatmap_cache)
              .unwrap_or_else(|err| {
                error!("Failed to select beatmap: {:?}", err);
              });
          }

          _ => {}
        }
      }

      ClientAction::KatOne if !repeat => {
        if self.game_state == LogicalState::Playing {
          self
            .gameplay_screen
            .hit(TaikoPlayerInput::KatOne, &core.graphics, &mut self.audio_engine, &self.app_state);
        }
      }

      ClientAction::KatTwo if !repeat => {
        if self.game_state == LogicalState::Playing {
          self
            .gameplay_screen
            .hit(TaikoPlayerInput::KatTwo, &core.graphics, &mut self.audio_engine, &self.app_state);
        }
      }

      ClientAction::DonOne if !repeat => {
        if self.game_state == LogicalState::Playing {
          self
            .gameplay_screen
            .hit(TaikoPlayerInput::DonOne, &core.graphics, &mut self.audio_engine, &self.app_state);
        }
      }

      ClientAction::DonTwo if !repeat => {
        if self.game_state == LogicalState::Playing {
          self
            .gameplay_screen
            .hit(TaikoPlayerInput::DonTwo, &core.graphics, &mut self.audio_engine, &self.app_state);
        }
      }

      _ => {}
    }
  }

  pub fn dispatch(&mut self, core: &mut Core<Self>, event: ClientEvent) {
    match event {
      ClientEvent::PickBeatmap { path } => {
        self.game_state = LogicalState::Playing;
        self.gameplay_screen.play(&path, &core.graphics, &mut self.audio_engine, &self.app_state);
      }

      ClientEvent::SelectBeatmap => {
        self.play_beatmap_audio();
      }

      ClientEvent::RetryBeatmap => {
        self.gameplay_screen.reset(&core.graphics, &mut self.audio_engine);
      }

      ClientEvent::ToggleSettings => {
        self.settings_screen.toggle_settings();
      }

      ClientEvent::ShowResultScreen { path } => {
        self.game_state = LogicalState::Results;
        self.result_screen.finish(&self.beatmap_cache, &path);
      }
    }
  }

  pub fn file(&mut self, _core: &mut Core<Self>, path: PathBuf, file: Vec<u8>) {
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

  pub fn play_beatmap_audio(&mut self) {
    use std::time::Duration;

    let selected = self.selection_screen.beatmap_selector().selected();
    let Some((path, beatmap)) = self.beatmap_cache.get_index(selected) else {
      return;
    };

    if beatmap.audio_path != self.prev_audio_path || path.parent().unwrap() != self.prev_beatmap_path {
      self.prev_beatmap_path = path.parent().unwrap().to_owned();
      self.prev_audio_path = beatmap.audio_path.clone();
    } else {
      return;
    }

    let audio_path = path.parent().unwrap().join(&beatmap.audio_path);
    let file = BufReader::new(File::open(audio_path).unwrap());
    let source = Decoder::new(file).unwrap();

    let config = self.audio_engine.device().default_output_config().unwrap();
    let source = UniformSourceIterator::new(source, config.channels(), config.sample_rate().0);

    // TODO: calculate length of the audio
    let length = source.total_duration().unwrap_or(Duration::from_secs(0));
    self.audio_engine.set_length(length.into());

    self.audio_engine.set_playing(false);
    self.audio_controller.play_audio(source);
    self.audio_engine.set_position(Time::from_ms(beatmap.preview_time as f64));
    self.audio_engine.set_playing(true);
  }
}
