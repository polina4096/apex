use std::{fs::File, io::BufReader, path::Path};

use rodio::{source::UniformSourceIterator, Decoder, DeviceTrait};

use crate::client::{
  audio::game_audio::{GameAudio, GameAudioController},
  client::Client,
  event::ClientEvent,
  gameplay::{
    beatmap::Beatmap,
    taiko_player::{BreakState, TaikoInput, TaikoPlayer},
  },
  graphics::taiko_renderer::taiko_renderer::{TaikoRenderer, TaikoRendererConfig},
  score::{judgement_processor::Judgement, score_processor::ScoreProcessor},
  settings::Settings,
  ui::{break_overlay::BreakOverlayView, ingame_overlay::IngameOverlayView},
};
use apex_framework::{
  audio::arc_buffer::ArcSamplesBuffer,
  core::Core,
  event::EventBus,
  graphics::{drawable::Drawable, graphics::Graphics},
  time::{clock::AbstractClock, time::Time},
};

pub struct GameplayScreen {
  audio_controller: GameAudioController,

  taiko_renderer: TaikoRenderer,
  ingame_overlay: IngameOverlayView,
  break_overlay: BreakOverlayView,

  taiko_player: TaikoPlayer,
  score_processor: ScoreProcessor,

  don_hitsound: ArcSamplesBuffer<f32>,
  kat_hitsound: ArcSamplesBuffer<f32>,
}

impl GameplayScreen {
  pub fn new(event_bus: EventBus<ClientEvent>, graphics: &Graphics, audio: &GameAudio, settings: &Settings) -> Self {
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
        // Apparently setting it to f64::INFINITY leads to a crash, see https://github.com/gfx-rs/wgpu/issues/6082
        hit_height: if settings.taiko.hit_animation() { 12.5 } else { 9999999.0 },
      },
    );

    let taiko_player = TaikoPlayer::new(event_bus.clone());
    let score_processor = ScoreProcessor::default();

    let don_hitsound = audio.load_sound("./assets/red.wav");
    let kat_hitsound = audio.load_sound("./assets/blue.wav");

    return Self {
      audio_controller: audio.controller(),

      taiko_renderer,
      ingame_overlay,
      break_overlay,

      score_processor,
      taiko_player,

      don_hitsound,
      kat_hitsound,
    };
  }

  pub fn hit(&mut self, input: TaikoInput, graphics: &Graphics, audio: &mut GameAudio) {
    let time = audio.position();

    match input {
      TaikoInput::DonOne | TaikoInput::DonTwo => {
        self.audio_controller.play_sound(self.don_hitsound.clone());
      }

      TaikoInput::KatOne | TaikoInput::KatTwo => {
        self.audio_controller.play_sound(self.kat_hitsound.clone());
      }
    }

    self.ingame_overlay.hit(input);

    if let Some((result, hit_idx)) = self.taiko_player.hit(time, input) {
      self.score_processor.feed(time, Some(input), result.judgement);
      self.ingame_overlay.update_last_hit_result(result.judgement);

      if result.judgement != Judgement::Miss {
        self.taiko_renderer.set_hit(&graphics.queue, hit_idx, time);
      }
    }
  }

  pub fn play(&mut self, beatmap_path: &Path, graphics: &Graphics, audio: &mut GameAudio) {
    let data = std::fs::read_to_string(beatmap_path).unwrap();
    let beatmap = Beatmap::parse(data);

    let config = audio.device().default_output_config().unwrap();
    let audio_path = beatmap_path.parent().unwrap().join(&beatmap.audio);
    let file = BufReader::new(File::open(audio_path).unwrap());
    let source = Decoder::new(file).unwrap();
    let source = UniformSourceIterator::new(source, config.channels(), config.sample_rate().0);

    let end_time = beatmap.hit_objects.last().unwrap().time;

    audio.set_playing(false);
    audio.set_source(source);
    audio.set_length(end_time);

    self.taiko_renderer.load_beatmap(&graphics.device, beatmap.clone());
    self.taiko_player.play(beatmap, beatmap_path.to_owned());
    std::mem::take(&mut self.score_processor);

    audio.set_position(Time::zero() - audio.lead_in);
    audio.set_playing(true);
  }

  pub fn reset(&mut self, graphics: &Graphics, audio: &mut GameAudio) {
    self.taiko_renderer.restart_beatmap(&graphics.queue);
    self.taiko_player.reset();

    std::mem::take(&mut self.score_processor);

    audio.set_playing(false);
    audio.set_position(Time::zero() - audio.lead_in);
    audio.set_playing(true);
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, audio: &mut GameAudio, settings: &Settings) {
    let time = audio.position();

    self.taiko_player.tick(time, audio, &mut self.score_processor, &mut self.ingame_overlay);

    self.taiko_renderer.prepare(&core.graphics.queue, time);
    self.ingame_overlay.prepare(core, audio, &self.score_processor, settings);

    let leniency = Time::from_ms(settings.gameplay.break_leniency_end() as f64);
    match self.taiko_player.is_break(time, leniency) {
      BreakState::Break(break_point) => {
        self.break_overlay.prepare(
          core,
          time,
          &break_point,
          Time::from_ms(settings.gameplay.break_leniency_start() as f64),
          Time::from_ms(settings.gameplay.break_leniency_end() as f64),
        );
      }

      BreakState::Intro(break_point) => {
        self.break_overlay.prepare(
          core,
          time,
          &break_point,
          Time::zero(),
          Time::from_ms(settings.gameplay.break_leniency_end() as f64),
        );
      }

      BreakState::None => {}
    }
  }

  pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
    self.taiko_renderer.render(rpass);
  }

  pub fn taiko_player(&mut self) -> &mut TaikoPlayer {
    return &mut self.taiko_player;
  }

  pub fn taiko_renderer(&mut self) -> &mut TaikoRenderer {
    return &mut self.taiko_renderer;
  }
}

impl Drawable for GameplayScreen {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.taiko_renderer.recreate(device, queue, format);
  }
}
