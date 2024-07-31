use std::{
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
};

use rodio::{
  source::{Empty, UniformSourceIterator},
  Decoder, DeviceTrait as _, Source,
};
use tap::Tap;
use winit::{
  event::{KeyEvent, Modifiers},
  keyboard::{KeyCode, PhysicalKey},
};

use crate::core::{
  app::App,
  audio::{self, audio_engine::AudioEngine, audio_mixer::AudioController},
  core::Core,
  data::persistent::Persistent as _,
  event::EventBus,
  graphics::drawable::Drawable,
  input::{
    action::AppActions as _,
    keybinds::{KeyCombination, Keybinds},
    Input,
  },
  time::{clock::AbstractClock, time::Time},
};

use super::{
  action::ClientAction,
  event::ClientEvent,
  gameplay::beatmap_cache::{BeatmapCache, BeatmapInfo},
  screen::{
    gameplay_screen::gameplay_screen::GameplayScreen, pause_screen::pause_screen::PauseScreen,
    recording_screen::recording_screen::RecordingScreen, result_screen::result_screen::ResultScreen,
    selection_screen::selection_screen::SelectionScreen, settings_screen::settings_screen::SettingsScreen,
  },
  settings::{proxy::ClientSettingsProxy, Settings},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
  Selection,
  Playing,
  Paused,
  Results,
}

pub struct Client {
  pub(crate) input: Input<ClientAction>,
  pub(crate) audio_engine: AudioEngine,
  pub(crate) audio_controller: AudioController,
  pub(crate) event_bus: EventBus<ClientEvent>,

  pub(crate) game_state: GameState,

  pub(crate) beatmap_cache: BeatmapCache,

  /// Configuration and state of the whole game
  pub(crate) settings: Settings,

  pub(crate) prev_audio_path: PathBuf,
  pub(crate) prev_beatmap_path: PathBuf,

  pub(crate) selection_screen: SelectionScreen,
  pub(crate) gameplay_screen: GameplayScreen,
  pub(crate) result_screen: ResultScreen,
  pub(crate) settings_screen: SettingsScreen,
  pub(crate) recording_screen: RecordingScreen,
  pub(crate) pause_screen: PauseScreen,
}

impl Drop for Client {
  fn drop(&mut self) {
    self.settings.save("./config.toml");
    self.input.keybinds.save("./keybinds.toml");
  }
}

impl App for Client {
  type Event = ClientEvent;

  fn prepare(&mut self, core: &mut Core<Self>, encoder: &mut wgpu::CommandEncoder) {
    core.egui_ctx.begin_frame(core.window);

    let beatmap_idx = self.selection_screen.beatmap_selector().selected();
    self.recording_screen.prepare(core, beatmap_idx, &self.beatmap_cache);

    self.settings_screen.prepare(
      core.egui_ctx.egui_ctx(),
      &mut self.input,
      &mut self.settings,
      &mut ClientSettingsProxy {
        proxy: &core.proxy,

        gameplay_screen: &mut self.gameplay_screen,
        audio_controller: &mut self.audio_controller,

        device: &core.graphics.device,
        queue: &core.graphics.queue,
        surface: &core.graphics.surface,
        config: &mut core.graphics.config,
      },
    );

    match self.game_state {
      GameState::Selection => {
        self.selection_screen.prepare(core, &self.beatmap_cache, &mut self.audio_engine);
      }

      GameState::Playing => {
        self.gameplay_screen.prepare(core, &mut self.audio_engine, &self.settings);
      }

      GameState::Paused => {
        self.gameplay_screen.prepare(core, &mut self.audio_engine, &self.settings);
        self.pause_screen.prepare(
          core,
          &mut self.audio_engine,
          &mut self.audio_controller,
          &mut self.gameplay_screen,
          &mut self.selection_screen,
          &self.beatmap_cache,
          &mut self.game_state,
          &self.settings,
        )
      }

      GameState::Results => {
        self.result_screen.prepare(core, &mut self.settings, &self.beatmap_cache);
      }
    }

    core.egui_ctx.end_frame(&core.graphics, encoder);
  }

  fn render<'rpass>(&'rpass self, core: &'rpass mut Core<Self>, rpass: &mut wgpu::RenderPass<'rpass>) {
    // Draw wgpu
    match self.game_state {
      GameState::Selection => {}

      GameState::Playing => {
        self.gameplay_screen.render(rpass);
      }

      GameState::Paused => {
        self.gameplay_screen.render(rpass);
      }

      GameState::Results => {}
    }

    // Draw egui
    core.egui_ctx.render(&core.graphics, rpass);
  }

  fn resize(&mut self, core: &mut Core<Self>, size: winit::dpi::PhysicalSize<u32>) {
    self.gameplay_screen.resize(&core.graphics.queue, size);
  }

  fn scale(&mut self, core: &mut Core<Self>, scale_factor: f64) {
    self.gameplay_screen.scale(&core.graphics.queue, scale_factor);
    self.selection_screen.scale(scale_factor);
  }
}

impl Drawable for Client {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.gameplay_screen.recreate(device, queue, format);
    self.selection_screen.recreate(device, queue, format);
  }
}

impl Client {
  pub fn new(core: &mut Core<Self>, settings: Settings, event_bus: EventBus<ClientEvent>) -> Self {
    let input = Input::with_keybinds(Keybinds::load("./keybinds.toml"));

    let (m, a, s) = (settings.audio.master_volume(), settings.audio.music_volume(), settings.audio.effect_volume());
    let (audio_mixer, audio_controller) = audio::mixer(Empty::new(), m, a, s);
    let mut audio_engine = AudioEngine::new();
    audio_engine.set_source(audio_mixer);

    let game_state = GameState::Selection;

    let beatmap_cache = BeatmapCache::new().tap_mut(|cache| {
      cache.load_beatmaps("./beatmaps");
    });

    #[rustfmt::skip] let selection_screen = SelectionScreen::new(event_bus.clone(), &beatmap_cache, &mut audio_engine, &core.graphics, &mut core.egui_ctx, &settings);
    #[rustfmt::skip] let result_screen = ResultScreen::new(event_bus.clone(), &beatmap_cache, &PathBuf::new());
    #[rustfmt::skip] let gameplay_screen = GameplayScreen::new(event_bus.clone(), &core.graphics, &audio_engine, audio_controller.clone(), &settings);
    #[rustfmt::skip] let settings_screen = SettingsScreen::new();
    #[rustfmt::skip] let recording_screen = RecordingScreen::new();
    #[rustfmt::skip] let pause_screen = PauseScreen::new(event_bus.clone());

    let prev_audio_path = PathBuf::new();
    let prev_beatmap_path = PathBuf::new();

    return Self {
      input,
      audio_engine,
      audio_controller,
      event_bus,
      game_state,
      settings,
      prev_audio_path,
      prev_beatmap_path,
      beatmap_cache,
      selection_screen,
      gameplay_screen,
      result_screen,
      settings_screen,
      recording_screen,
      pause_screen,
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
      let mut captured = false;

      let comb = KeyCombination::new(event.physical_key, self.input.state.modifiers);
      if let Some(action) = self.input.keybinds.get(&comb).map(|x| x.id) {
        captured = action.execute(self, core, event.repeat);
      }

      if !captured {
        // Handle typing in selection screen
        if self.game_state == GameState::Selection {
          match event.physical_key {
            PhysicalKey::Code(KeyCode::Backspace) => {
              if self.selection_screen.beatmap_selector().has_query() {
                self.selection_screen.beatmap_selector_mut().pop_query();
                return;
              }
            }

            _ => {
              if let Some(c) = event.logical_key.to_text().and_then(|x| x.chars().next()) {
                self.selection_screen.beatmap_selector_mut().push_query(c);
              }
            }
          }
        }
      }
    }
  }

  pub fn modifiers(&mut self, modifiers: Modifiers) {
    self.input.state.modifiers = modifiers.state();
  }

  pub fn dispatch(&mut self, core: &mut Core<Self>, event: ClientEvent) {
    match event {
      ClientEvent::PickBeatmap { path } => {
        self.game_state = GameState::Playing;
        self.gameplay_screen.play(&path, &core.graphics, &mut self.audio_engine, &self.settings);
      }

      ClientEvent::SelectBeatmap => {
        self.play_beatmap_audio();
      }

      ClientEvent::RetryBeatmap => {
        self.gameplay_screen.reset(&core.graphics, &mut self.audio_engine);
      }

      ClientEvent::ToggleSettings => {
        self.settings_screen.toggle();
      }

      ClientEvent::ShowResultScreen { path } => {
        self.game_state = GameState::Results;
        self.result_screen.finish(&self.beatmap_cache, &path);
      }

      ClientEvent::ToggleRecordingWindow => {
        if !self.recording_screen.is_open() {
          self.recording_screen.toggle();
        }
      }
    }
  }

  pub fn file(&mut self, _core: &mut Core<Self>, path: PathBuf, file: Vec<u8>) {
    // TODO: this logic should be moved to the beatmap manager or whatever
    // TODO: properly parse beatmapset id
    let beatmapset_id = path.file_name().unwrap().to_str().unwrap().split_whitespace().next().unwrap();
    zip::read::ZipArchive::new(std::io::Cursor::new(file))
      .unwrap()
      .extract(format!("./beatmaps/{}", beatmapset_id))
      .unwrap();

    self.beatmap_cache.load_difficulties(format!("./beatmaps/{}", beatmapset_id));
  }

  pub fn play_beatmap_audio(&mut self) {
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

    Self::play_beatmap_audio_unchecked(&mut self.audio_engine, &mut self.audio_controller, path, beatmap);
  }

  pub fn play_beatmap_audio_unchecked(
    audio_engine: &mut AudioEngine,
    audio_controller: &mut AudioController,
    path: &Path,
    beatmap: &BeatmapInfo,
  ) {
    use std::time::Duration;

    let audio_path = path.parent().unwrap().join(&beatmap.audio_path);
    let file = BufReader::new(File::open(audio_path).unwrap());
    let source = Decoder::new(file).unwrap();

    let config = audio_engine.device().default_output_config().unwrap();
    let source = UniformSourceIterator::new(source, config.channels(), config.sample_rate().0);

    // TODO: calculate length of the audio
    let length = source.total_duration().unwrap_or(Duration::from_secs(0));
    audio_engine.set_length(length.into());

    audio_engine.set_playing(false);
    audio_controller.play_audio(source);
    audio_engine.set_position(Time::from_ms(beatmap.preview_time as f64));
    audio_engine.set_playing(true);
  }
}
