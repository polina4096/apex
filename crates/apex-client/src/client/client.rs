use std::{
  fs::File,
  io::BufReader,
  num::NonZero,
  path::{Path, PathBuf},
  sync::atomic::AtomicBool,
};

use pollster::FutureExt as _;
use rodio::{
  source::{Empty, UniformSourceIterator},
  Decoder, DeviceTrait as _, Source,
};
use rusqlite::Connection;
use tap::Tap;
use triomphe::Arc;
use winit::{
  event::{KeyEvent, Modifiers},
  event_loop::{ActiveEventLoop, EventLoopProxy},
  keyboard::{KeyCode, PhysicalKey},
  window::Window,
};

use apex_framework::{
  app::App,
  audio::{self, audio_engine::AudioEngine},
  core::Core,
  data::persistent::Persistent as _,
  event::{CoreEvent, EventBus},
  graphics::{
    drawable::Drawable,
    graphics::Graphics,
    presentation::{frame_limiter::FrameLimiter, frame_sync::FrameSync},
  },
  input::{
    action::AppActions as _,
    keybinds::{KeyCombination, Keybinds},
    Input,
  },
  time::{clock::AbstractClock, time::Time},
};

use super::{
  action::ClientAction,
  audio::game_audio::GameAudio,
  event::ClientEvent,
  gameplay::beatmap_cache::{BeatmapCache, BeatmapInfo},
  graphics::{FrameLimiterOptions, RenderingBackend},
  score::score_cache::ScoreCache,
  screen::{
    debug_screen::debug_screen::DebugScreen, gameplay_screen::gameplay_screen::GameplayScreen,
    pause_screen::pause_screen::PauseScreen, recording_screen::recording_screen::RecordingScreen,
    result_screen::result_screen::ResultScreen, selection_screen::selection_screen::SelectionScreen,
    settings_screen::settings_screen::SettingsScreen,
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
  pub(crate) audio: GameAudio,
  pub(crate) event_bus: EventBus<ClientEvent>,

  pub(crate) game_state: GameState,

  pub(crate) settings: Settings,

  pub(crate) beatmap_cache: BeatmapCache,
  pub(crate) score_cache: ScoreCache,

  pub(crate) prev_audio_path: PathBuf,
  pub(crate) prev_beatmap_path: PathBuf,

  pub(crate) selection_screen: SelectionScreen,
  pub(crate) gameplay_screen: GameplayScreen,
  pub(crate) result_screen: ResultScreen,
  pub(crate) settings_screen: SettingsScreen,
  pub(crate) recording_screen: RecordingScreen,
  pub(crate) pause_screen: PauseScreen,
  pub(crate) debug_screen: DebugScreen,
}

impl App for Client {
  type Event = ClientEvent;

  fn create(
    event_loop: &ActiveEventLoop,
    window: Arc<Window>,
    app_focus: Arc<AtomicBool>,
    proxy: EventLoopProxy<CoreEvent<Self::Event>>,
  ) -> (Self, Core<Self>) {
    let settings = Settings::load("./config.toml");

    #[allow(clippy::infallible_destructuring_match)]
    let backend = match settings.graphics.rendering_backend() {
      RenderingBackend::Wgpu(wgpu_backend) => wgpu_backend,
    };

    let graphics = Graphics::new(
      &window,
      backend.into(),
      settings.graphics.present_mode().into(),
      settings.graphics.max_frame_latency(),
    )
    .block_on();

    let client = Client::new(&graphics, settings, EventBus::new(proxy.clone()));
    let mut core = Core::new(event_loop, proxy, window.clone(), app_focus, graphics);

    // Setup external frame synchronization
    core.frame_sync.set_current_window(window);

    // Setup frame limiter
    reconfigure_frame_sync(&mut core.frame_limiter, &mut core.frame_sync, client.settings.graphics.frame_limiter());

    return (client, core);
  }

  fn recreate_graphics(&mut self, core: &mut Core<Self>) -> Graphics {
    #[allow(clippy::infallible_destructuring_match)]
    let backend = match self.settings.graphics.rendering_backend() {
      RenderingBackend::Wgpu(wgpu_backend) => wgpu_backend,
    };

    return Graphics::new(
      &core.window,
      backend.into(),
      self.settings.graphics.present_mode().into(),
      self.settings.graphics.max_frame_latency(),
    )
    .block_on();
  }

  fn destroy(&self) {
    self.settings.save("./config.toml");
    self.input.keybinds.save("./keybinds.toml");
  }

  fn prepare(&mut self, core: &mut Core<Self>, encoder: &mut wgpu::CommandEncoder) {
    core.egui.begin_frame(&core.window);

    let beatmap_idx = self.selection_screen.beatmap_selector().selected();
    self.recording_screen.prepare(core, beatmap_idx, &self.beatmap_cache);

    self.settings_screen.prepare(
      core.egui.ctx(),
      &mut self.input,
      &mut self.settings,
      &mut ClientSettingsProxy {
        proxy: &core.proxy,

        frame_limiter: &mut core.frame_limiter,
        frame_sync: &mut core.frame_sync,
        gameplay_screen: &mut self.gameplay_screen,
        audio: &mut self.audio,

        device: &core.graphics.device,
        queue: &core.graphics.queue,
        surface: &core.graphics.surface,
        config: &mut core.graphics.config,
      },
    );

    self.debug_screen.prepare(core);

    match self.game_state {
      GameState::Selection => {
        self.selection_screen.prepare(core, &self.beatmap_cache, &mut self.score_cache, &mut self.audio);
      }

      GameState::Playing => {
        self.gameplay_screen.prepare(core, &mut self.audio, &self.settings);
      }

      GameState::Paused => {
        self.gameplay_screen.prepare(core, &mut self.audio, &self.settings);
        self.pause_screen.prepare(
          core,
          &mut self.audio,
          &mut self.selection_screen,
          &self.beatmap_cache,
          &mut self.game_state,
          &self.settings,
        )
      }

      GameState::Results => {
        self.result_screen.prepare(core, &self.beatmap_cache, &self.score_cache);
      }
    }

    core.egui.end_frame(&core.window, &core.graphics, encoder);
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
    core.egui.render(&core.graphics, rpass);
  }

  fn resize(&mut self, core: &mut Core<Self>, size: winit::dpi::PhysicalSize<u32>) {
    self.gameplay_screen.taiko_renderer().resize(&core.graphics.queue, size.width, size.height);
  }

  fn scale(&mut self, core: &mut Core<Self>, scale_factor: f64) {
    self.gameplay_screen.taiko_renderer().scale(&core.graphics.queue, scale_factor);
    self.selection_screen.scale(scale_factor);
  }

  fn input(&mut self, core: &mut Core<Self>, event: KeyEvent) {
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

  fn modifiers(&mut self, modifiers: Modifiers) {
    self.input.state.modifiers = modifiers.state();
  }

  fn dispatch(&mut self, core: &mut Core<Self>, event: ClientEvent) {
    match event {
      ClientEvent::PickBeatmap { path } => {
        self.game_state = GameState::Playing;
        self.gameplay_screen.play(&path, &core.graphics, &mut self.audio);
      }

      ClientEvent::SelectBeatmap => {
        self.play_beatmap_audio();
      }

      ClientEvent::RetryBeatmap => {
        self.gameplay_screen.reset(&core.graphics, &mut self.audio);
      }

      ClientEvent::ToggleSettings => {
        self.settings_screen.toggle();
      }

      ClientEvent::ShowResultScreen { path, score } => {
        let score_id = self.score_cache.insert(path.clone(), score);
        self.result_screen.set_score(&self.beatmap_cache, &self.score_cache, &path, score_id);
        self.selection_screen.update_scores(&mut self.score_cache, &path);
        self.game_state = GameState::Results;
      }

      ClientEvent::ViewScore { path, score_id } => {
        self.result_screen.set_score(&self.beatmap_cache, &self.score_cache, &path, score_id);
        self.game_state = GameState::Results;
      }

      ClientEvent::ToggleRecordingWindow => {
        if !self.recording_screen.is_open() {
          self.recording_screen.toggle();
        }
      }
    }
  }

  fn file_dropped(&mut self, _core: &mut Core<Self>, path: PathBuf, file: Vec<u8>) {
    // TODO: this logic should be moved to the beatmap manager or whatever
    // TODO: properly parse beatmapset id
    let beatmapset_id = path.file_name().unwrap().to_str().unwrap().split_whitespace().next().unwrap();
    zip::read::ZipArchive::new(std::io::Cursor::new(file))
      .unwrap()
      .extract(format!("./beatmaps/{}", beatmapset_id))
      .unwrap();

    self.beatmap_cache.load_difficulties(format!("./beatmaps/{}", beatmapset_id));
  }
}

impl Drawable for Client {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.gameplay_screen.recreate(device, queue, format);
    self.selection_screen.recreate(device, queue, format);
  }
}

impl Client {
  pub fn new(graphics: &Graphics, settings: Settings, event_bus: EventBus<ClientEvent>) -> Self {
    let input = Input::with_keybinds(Keybinds::load("./keybinds.toml"));

    let (m, a, s) = (settings.audio.master_volume(), settings.audio.music_volume(), settings.audio.effects_volume());
    let (audio_mixer, audio_controller) = audio::mixer(Empty::new(), m, a, s);
    let audio_engine = AudioEngine::new().tap_mut(|x| x.set_source(audio_mixer));
    let mut audio = GameAudio::new(audio_engine, audio_controller)
      .with_lead_in(Time::from_ms(settings.gameplay.lead_in() as f64))
      .with_lead_out(Time::from_ms(settings.gameplay.lead_out() as f64));

    let game_state = GameState::Selection;

    let beatmap_cache = BeatmapCache::new().tap_mut(|cache| {
      cache.load_beatmaps("./beatmaps");
    });

    let conn = Connection::open("./scores.db").unwrap();
    let score_cache = ScoreCache::new(conn);

    #[rustfmt::skip] let selection_screen = SelectionScreen::new(event_bus.clone(), &beatmap_cache, &mut audio, graphics, &settings);
    #[rustfmt::skip] let result_screen = ResultScreen::new(event_bus.clone(), &score_cache);
    #[rustfmt::skip] let gameplay_screen = GameplayScreen::new(event_bus.clone(), graphics, &audio, &settings);
    #[rustfmt::skip] let settings_screen = SettingsScreen::new();
    #[rustfmt::skip] let recording_screen = RecordingScreen::new();
    #[rustfmt::skip] let pause_screen = PauseScreen::new(event_bus.clone());
    #[rustfmt::skip] let debug_screen = DebugScreen::new();

    let prev_audio_path = PathBuf::new();
    let prev_beatmap_path = PathBuf::new();

    return Self {
      input,
      audio,
      event_bus,
      game_state,
      settings,
      prev_audio_path,
      prev_beatmap_path,
      beatmap_cache,
      score_cache,
      selection_screen,
      gameplay_screen,
      result_screen,
      settings_screen,
      recording_screen,
      pause_screen,
      debug_screen,
    };
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

    Self::play_beatmap_audio_unchecked(&mut self.audio, path, beatmap);
  }

  pub fn play_beatmap_audio_unchecked(audio: &mut GameAudio, path: &Path, beatmap: &BeatmapInfo) {
    use std::time::Duration;

    let audio_path = path.parent().unwrap().join(&beatmap.audio_path);
    let file = BufReader::new(File::open(audio_path).unwrap());
    let source = Decoder::new(file).unwrap();

    let config = audio.device().default_output_config().unwrap();
    let source = UniformSourceIterator::new(source, config.channels(), config.sample_rate().0);

    // TODO: calculate length of the audio
    let length = source.total_duration().unwrap_or(Duration::from_secs(0));
    audio.set_length(length.into());

    audio.set_playing(false);
    audio.set_source(source);
    audio.set_position(Time::from_ms(beatmap.preview_time as f64));
    audio.set_playing(true);
  }
}

pub fn reconfigure_frame_sync(
  frame_limiter: &mut FrameLimiter,
  frame_sync: &mut FrameSync,
  options: FrameLimiterOptions,
) {
  match options {
    FrameLimiterOptions::Custom(fps) => {
      frame_sync.disable_external_sync();

      frame_limiter.set_enabled(true);
      frame_limiter.set_target_fps(Some(NonZero::new(fps as u16).unwrap()));
    }

    FrameLimiterOptions::DisplayLink => {
      frame_limiter.set_enabled(false);

      frame_sync.enable_external_sync().unwrap();
    }

    FrameLimiterOptions::Unlimited => {
      frame_sync.disable_external_sync();

      frame_limiter.set_enabled(true);
      frame_limiter.set_target_fps(None);
    }
  }
}
