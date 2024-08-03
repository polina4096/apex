use std::{
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
};

use jiff::Timestamp;
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
      beatmap::{Beatmap, BreakPoint},
      beatmap_audio::BeatmapAudio,
      taiko_player::{TaikoPlayer, TaikoPlayerInput},
    },
    graphics::taiko_renderer::taiko_renderer::{TaikoRenderer, TaikoRendererConfig},
    score::score_processor::{ScoreProcessor, ScoreProcessorEvent},
    settings::Settings,
    ui::{
      break_overlay::BreakOverlayView,
      ingame_overlay::{HitResult, IngameOverlayView},
    },
  },
  core::{
    audio::{audio_engine::AudioEngine, audio_mixer::AudioController},
    core::Core,
    event::EventBus,
    graphics::{color::Color, drawable::Drawable, graphics::Graphics},
    time::{clock::AbstractClock, time::Time},
  },
};

pub struct GameplayScreen {
  audio_controller: AudioController,
  event_bus: EventBus<ClientEvent>,

  taiko_renderer: TaikoRenderer,
  ingame_overlay: IngameOverlayView,
  break_overlay: BreakOverlayView,

  channels: ChannelCount,
  sample_rate: SampleRate,
  don_hitsound: Vec<f32>,
  kat_hitsound: Vec<f32>,

  beatmap_path: PathBuf,
  play_date: Timestamp,

  beatmap: Option<Beatmap>,
  score_processor: ScoreProcessor,
  player: TaikoPlayer,
  audio: BeatmapAudio,
}

impl GameplayScreen {
  pub fn new(
    event_bus: EventBus<ClientEvent>,
    graphics: &Graphics,
    audio_engine: &AudioEngine,
    audio_controller: AudioController,
    settings: &Settings,
  ) -> Self {
    let ingame_overlay = IngameOverlayView::new();
    let break_overlay = BreakOverlayView::new();
    let taiko_renderer = TaikoRenderer::new(
      &graphics.device,
      &graphics.queue,
      graphics.config.format,
      TaikoRendererConfig {
        width: graphics.size.width,
        height: graphics.size.height,
        scale_factor: graphics.scale,
        scale: settings.taiko.scale(),
        zoom: settings.taiko.zoom(),
        hit_position_x: settings.taiko.hit_position_x(),
        hit_position_y: settings.taiko.hit_position_y(),
        don: settings.taiko.don_color(),
        kat: settings.taiko.kat_color(),
        hit_height: if settings.taiko.hit_animation() { 12.5 } else { 9999.0 },
      },
    );

    let beatmap_path = PathBuf::new();
    let play_date = Timestamp::default();

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
      break_overlay,

      channels,
      sample_rate,
      don_hitsound,
      kat_hitsound,

      beatmap_path,
      play_date,

      beatmap,
      score_processor: score,
      player,
      audio,
    };
  }

  pub fn hit(&mut self, input: TaikoPlayerInput, graphics: &Graphics, audio: &mut AudioEngine) {
    let Some(beatmap) = &self.beatmap else { return };
    let mut audio = self.audio.borrow(audio);
    let time = audio.position();

    self.ingame_overlay.hit(input);

    match input {
      TaikoPlayerInput::DonOne | TaikoPlayerInput::DonTwo => {
        let source = SamplesBuffer::<f32>::new(self.channels, self.sample_rate.0, self.don_hitsound.clone());

        self.audio_controller.play_sound(source);
      }

      TaikoPlayerInput::KatOne | TaikoPlayerInput::KatTwo => {
        let source = SamplesBuffer::<f32>::new(self.channels, self.sample_rate.0, self.kat_hitsound.clone());

        self.audio_controller.play_sound(source);
      }
    }

    self.player.hit(time, input, beatmap, |result, idx| {
      self.score_processor.feed(ScoreProcessorEvent { result });
      self.taiko_renderer.set_hit(&graphics.queue, time, idx);
      self.ingame_overlay.update_last_hit_result(result);
    });
  }

  pub fn reset(&mut self, graphics: &Graphics, audio: &mut AudioEngine) {
    self.taiko_renderer.restart_beatmap(&graphics.queue);
    self.player.reset();

    std::mem::take(&mut self.score_processor);
    self.play_date = Timestamp::now();

    let mut audio = self.audio.borrow(audio);

    audio.set_playing(false);
    audio.set_position(Time::zero() - audio.lead_in);
    audio.set_playing(true);
  }

  pub fn play(&mut self, beatmap_path: &Path, graphics: &Graphics, audio: &mut AudioEngine, settings: &Settings) {
    let data = std::fs::read_to_string(beatmap_path).unwrap();
    let beatmap = Beatmap::parse(data);
    let end_time = beatmap.hit_objects.last().unwrap().time;

    self.taiko_renderer.load_beatmap(&graphics.device, beatmap.clone());
    std::mem::take(&mut self.score_processor);
    self.player.reset();

    self.beatmap_path = beatmap_path.to_owned();

    let audio_path = beatmap_path.parent().unwrap().join(&beatmap.audio);
    let file = BufReader::new(File::open(audio_path).unwrap());
    let source = Decoder::new(file).unwrap().delay(std::time::Duration::from_millis(settings.gameplay.lead_in()));

    let config = audio.device().default_output_config().unwrap();
    let source = UniformSourceIterator::new(source, config.channels(), config.sample_rate().0);

    let mut audio = self.audio.borrow(audio);

    audio.set_playing(false);
    audio.set_source(source);
    audio.set_length(end_time);

    self.beatmap = Some(beatmap);
    self.play_date = Timestamp::now();

    // audio.set_position(Time::zero() - audio.lead_in);
    audio.set_clock_position(Time::zero());
    audio.set_source_position(Time::zero());
    audio.set_playing(true);
  }

  pub fn set_paused(&mut self, state: bool, audio: &mut AudioEngine) {
    let mut audio = self.audio.borrow(audio);

    if state {
      audio.set_playing(false);
    } else {
      audio.set_playing(true);
    }
  }

  pub fn skip_break(&mut self, audio: &mut AudioEngine, break_leniency_end: Time) {
    // TOOD: error handling
    if let Some(beatmap) = &self.beatmap {
      let lead_in = self.audio.lead_in;
      let audio_offset = self.audio.audio_offset;
      let mut audio = self.audio.borrow(audio);
      let time = audio.position();

      let Some(obj) = beatmap.hit_objects.first() else {
        return;
      };

      if time < obj.time - break_leniency_end && obj.time > Time::from_seconds(10.0) {
        let point = BreakPoint { start: Time::zero(), end: obj.time };
        let delay_compensation = time - audio_offset;
        let pos = point.end - break_leniency_end;

        if delay_compensation < Time::zero() {
          audio.set_source_position(pos - delay_compensation + lead_in);
          audio.set_clock_position(pos + lead_in);
        } else {
          audio.set_position(pos);
        }
      }

      // TODO: optimize
      let p = beatmap.break_points.iter().find(|x| time >= x.start && time < x.end);

      if let Some(break_point) = p {
        audio.set_playing(false);
        // TOOD: make this a setting
        let break_leniency = Time::from_seconds(1.0);

        audio.set_position(break_point.end - break_leniency);
        audio.set_playing(true);
      }
    }
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, audio: &mut AudioEngine, settings: &Settings) {
    let mut audio = self.audio.borrow(audio);
    let time = audio.position();

    // delay after the last hit object before result screen
    if time >= audio.length() + audio.lead_out {
      let path = self.beatmap_path.clone();
      let score = self.score_processor.export(self.play_date, "player".to_string());
      self.event_bus.send(ClientEvent::ShowResultScreen { path, score });
    }

    if let Some(beatmap) = &self.beatmap {
      self.player.tick(time, beatmap, |_idx| {
        self.score_processor.feed(ScoreProcessorEvent { result: HitResult::Miss });
        self.ingame_overlay.update_last_hit_result(HitResult::Miss);
      });
    }

    self.taiko_renderer.prepare(&core.graphics.queue, time);
    self.ingame_overlay.prepare(core, &mut audio, &self.score_processor, settings);

    if let Some(beatmap) = &self.beatmap {
      let Some(obj) = beatmap.hit_objects.first() else {
        return;
      };

      if obj.time > Time::from_seconds(10.0) {
        let point = BreakPoint { start: Time::zero(), end: obj.time };
        self.break_overlay.prepare(core, time, &point, Time::zero(), Time::from_seconds(1.0));
      } else {
        // TODO: optimize
        let p = beatmap.break_points.iter().find(|x| time >= x.start && time < x.end);

        if let Some(break_point) = p {
          self
            .break_overlay
            .prepare(core, time, break_point, Time::from_seconds(1.0), Time::from_seconds(1.0));
        }
      }
    }
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

  pub fn set_taiko_zoom(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, zoom: f64) {
    self.taiko_renderer.set_zoom(device, queue, zoom);
  }

  pub fn set_taiko_scale(&mut self, queue: &wgpu::Queue, scale: f64) {
    self.taiko_renderer.set_scale(queue, scale);
  }

  pub fn set_taiko_hit_position_x(&mut self, queue: &wgpu::Queue, value: f32) {
    self.taiko_renderer.set_hit_position_x(queue, value);
  }

  pub fn set_taiko_hit_position_y(&mut self, queue: &wgpu::Queue, value: f32) {
    self.taiko_renderer.set_hit_position_y(queue, value);
  }

  pub fn set_taiko_don_color(&mut self, device: &wgpu::Device, color: Color) {
    self.taiko_renderer.set_don_color(device, color);
  }

  pub fn set_taiko_kat_color(&mut self, device: &wgpu::Device, color: Color) {
    self.taiko_renderer.set_kat_color(device, color);
  }

  pub fn set_taiko_hit_animation(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat, value: bool) {
    self.taiko_renderer.set_hit_height(device, format, if value { 12.5 } else { 9999.0 });
  }

  pub fn set_audio_offset(&mut self, offset: Time) {
    self.audio.audio_offset = offset;
  }

  pub fn set_audio_lead_in(&mut self, lead_in: Time) {
    self.audio.lead_in = lead_in;
  }

  pub fn set_audio_lead_out(&mut self, lead_out: Time) {
    self.audio.lead_out = lead_out;
  }

  pub fn audio(&mut self) -> &mut BeatmapAudio {
    return &mut self.audio;
  }
}

impl Drawable for GameplayScreen {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.taiko_renderer.recreate(device, queue, format);
  }
}
