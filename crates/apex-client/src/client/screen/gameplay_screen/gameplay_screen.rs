use std::{
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
};

use rodio::{
  buffer::SamplesBuffer,
  cpal::{ChannelCount, SampleRate},
  source::UniformSourceIterator,
  Decoder, DeviceTrait, Source as _,
};
use tap::Tap;

use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    gameplay::{
      beatmap::Beatmap,
      beatmap_audio::BeatmapAudio,
      score_processor::{ScoreProcessor, ScoreProcessorEvent},
      taiko_player::{TaikoPlayer, TaikoPlayerInput},
    },
    graphics::taiko_renderer::taiko_renderer::{TaikoRenderer, TaikoRendererConfig},
    state::AppState,
    ui::ingame_overlay::{HitResult, IngameOverlayView},
  },
  core::{
    audio::{audio_engine::AudioEngine, audio_mixer::AudioController},
    core::Core,
    event::EventBus,
    graphics::{drawable::Drawable, graphics::Graphics},
    time::{clock::AbstractClock, time::Time},
  },
};

pub struct GameplayScreen {
  audio_controller: AudioController,
  event_bus: EventBus<ClientEvent>,

  taiko_renderer: TaikoRenderer,
  ingame_overlay: IngameOverlayView,

  channels: ChannelCount,
  sample_rate: SampleRate,
  don_hitsound: Vec<f32>,
  kat_hitsound: Vec<f32>,

  beatmap_path: PathBuf,

  beatmap: Option<Beatmap>,
  score: ScoreProcessor,
  player: TaikoPlayer,
  audio: BeatmapAudio,
}

impl GameplayScreen {
  pub fn new(
    event_bus: EventBus<ClientEvent>,
    graphics: &Graphics,
    audio_engine: &AudioEngine,
    audio_controller: AudioController,
    state: &AppState,
  ) -> Self {
    let ingame_overlay = IngameOverlayView::new();
    let taiko_renderer = TaikoRenderer::new(
      graphics,
      TaikoRendererConfig {
        width: graphics.size.width,
        height: graphics.size.height,
        scale_factor: graphics.scale,
        scale: state.taiko.scale,
        zoom: state.taiko.zoom,
        hit_position_x: state.taiko.hit_position_x,
        hit_position_y: state.taiko.hit_position_y,
        don: state.taiko.don_color,
        kat: state.taiko.kat_color,
      },
    );

    let beatmap_path = PathBuf::new();

    let beatmap = None;
    let score = ScoreProcessor::default();
    let player = TaikoPlayer::new();
    let audio = BeatmapAudio::new(audio_controller.clone()).tap_mut(|x| {
      x.lead_in = Time::from_ms(1000);
      x.lead_out = Time::from_ms(1000);
    });

    let config = audio_engine.device().default_output_config().unwrap();
    let channels = config.channels();
    let sample_rate = config.sample_rate();

    let source = Decoder::new(BufReader::new(File::open("./assets/red.wav").unwrap())).unwrap();
    let source = UniformSourceIterator::new(source, config.channels(), config.sample_rate().0);
    let don_hitsound = source.collect::<Vec<_>>();

    let source = Decoder::new(BufReader::new(File::open("./assets/blue.wav").unwrap())).unwrap();
    let source = UniformSourceIterator::new(source, config.channels(), config.sample_rate().0);
    let kat_hitsound = source.collect::<Vec<_>>();

    return Self {
      event_bus,
      audio_controller,

      taiko_renderer,
      ingame_overlay,

      channels,
      sample_rate,
      don_hitsound,
      kat_hitsound,

      beatmap_path,

      beatmap,
      score,
      player,
      audio,
    };
  }

  pub fn hit(&mut self, input: TaikoPlayerInput, graphics: &Graphics, audio: &mut AudioEngine, state: &AppState) {
    let Some(beatmap) = &self.beatmap else { return };
    let mut audio = self.audio.borrow(audio);
    let time = audio.position();

    self.ingame_overlay.hit(input);

    match input {
      TaikoPlayerInput::DonOne | TaikoPlayerInput::DonTwo => {
        let source = SamplesBuffer::<f32>::new(self.channels, self.sample_rate.0, self.don_hitsound.clone());

        if state.taiko.hitsounds {
          self.audio_controller.play_sound(source);
        }
      }

      TaikoPlayerInput::KatOne | TaikoPlayerInput::KatTwo => {
        let source = SamplesBuffer::<f32>::new(self.channels, self.sample_rate.0, self.kat_hitsound.clone());

        if state.taiko.hitsounds {
          self.audio_controller.play_sound(source);
        }
      }
    }

    self.player.hit(time, input, beatmap, |result, idx| {
      self.score.feed(ScoreProcessorEvent { result });
      self.taiko_renderer.set_hit(&graphics.queue, time, idx);
      self.ingame_overlay.show_hit_result(result);
    });
  }

  pub fn reset(&mut self, graphics: &Graphics, audio: &mut AudioEngine) {
    self.taiko_renderer.restart_beatmap(&graphics.queue);
    self.player.reset();

    std::mem::take(&mut self.score);

    let mut audio = self.audio.borrow(audio);
    audio.set_playing(false);
    audio.set_position(Time::zero());
    audio.set_playing(true);
  }

  pub fn play(&mut self, beatmap_path: &Path, graphics: &Graphics, audio: &mut AudioEngine, state: &AppState) {
    let data = std::fs::read_to_string(beatmap_path).unwrap();
    let beatmap = Beatmap::from(data);
    let end_time = beatmap.hit_objects.last().unwrap().time;

    self.taiko_renderer.load_beatmap(&graphics.device, beatmap.clone());
    std::mem::take(&mut self.score);
    self.player.reset();

    self.beatmap_path = beatmap_path.to_owned();

    let audio_path = beatmap_path.parent().unwrap().join(&beatmap.audio);
    let file = BufReader::new(File::open(audio_path).unwrap());
    let source = Decoder::new(file).unwrap().delay(std::time::Duration::from_millis(state.gameplay.lead_in));

    let config = audio.device().default_output_config().unwrap();
    let source = UniformSourceIterator::new(source, config.channels(), config.sample_rate().0);

    let mut audio = self.audio.borrow(audio);

    audio.set_playing(false);
    audio.set_source(source);
    audio.set_length(end_time);

    self.beatmap = Some(beatmap);

    audio.set_position(Time::zero());
    audio.set_playing(true);
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, audio: &mut AudioEngine, state: &AppState) {
    let mut audio = self.audio.borrow(audio);
    let time = audio.position();

    if audio.lead_in.to_ms() != state.gameplay.lead_in as i64 {
      audio.lead_in = Time::from_ms(state.gameplay.lead_in as f64);
    }

    if audio.lead_out.to_ms() != state.gameplay.lead_out as i64 {
      audio.lead_out = Time::from_ms(state.gameplay.lead_out as f64);
    }

    if audio.audio_offset.to_ms() != state.gameplay.audio_offset {
      audio.audio_offset = Time::from_ms(state.gameplay.audio_offset as f64);
    }

    // delay after the last hit object before result screen
    if time >= audio.length() + audio.lead_out {
      let path = self.beatmap_path.clone();
      self.event_bus.send(ClientEvent::ShowResultScreen { path });
    }

    if let Some(beatmap) = &self.beatmap {
      self.player.tick(time, beatmap, |_idx| {
        self.score.feed(ScoreProcessorEvent { result: HitResult::Miss });
        self.ingame_overlay.show_hit_result(HitResult::Miss);
      });
    }

    self.taiko_renderer.prepare(&core.graphics.queue, time);
    self.ingame_overlay.prepare(core, &mut audio, &self.score, state);
  }

  pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
    self.taiko_renderer.render(rpass);
  }

  pub fn resize(&mut self, queue: &wgpu::Queue, size: winit::dpi::PhysicalSize<u32>) {
    self.taiko_renderer.resize(queue, size.width, size.height);
  }

  pub fn scale(&mut self, queue: &wgpu::Queue, scale_factor: f64) {
    self.taiko_renderer.scale(queue, scale_factor);
  }
}

impl Drawable for GameplayScreen {
  fn recreate(&mut self, graphics: &Graphics) {
    self.taiko_renderer.recreate(graphics);
  }
}
