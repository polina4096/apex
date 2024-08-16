use std::{fs::File, io::BufReader, path::Path};

use glam::{vec2, Vec2};
use jiff::Timestamp;
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
  graphics::{
    color::Color, drawable::Drawable, graphics::Graphics, origin::Origin,
    sprite_renderer::sprite_renderer::SpriteRenderer,
  },
  time::{clock::AbstractClock, time::Time},
};

use super::{hit_drum_display::HitDrumDisplay, hit_result_display::HitResultDisplay};

pub struct GameplayScreen {
  event_bus: EventBus<ClientEvent>,
  audio_controller: GameAudioController,

  taiko_renderer: TaikoRenderer,
  sprite_renderer: SpriteRenderer,

  hit_result_display: HitResultDisplay,
  hit_drum_display: HitDrumDisplay,
  ingame_overlay: IngameOverlayView,
  break_overlay: BreakOverlayView,

  taiko_player: TaikoPlayer,
  score_processor: ScoreProcessor,

  don_hitsound: ArcSamplesBuffer<f32>,
  kat_hitsound: ArcSamplesBuffer<f32>,

  hit_pos_sprite: usize,

  // Hit pos settings
  hit_position_x_px: f32,
  hit_position_y_px: f32,
  hit_position_y_perc: f32,
  gameplay_scale: f32,
}

impl GameplayScreen {
  pub fn new(event_bus: EventBus<ClientEvent>, graphics: &Graphics, audio: &GameAudio, settings: &Settings) -> Self {
    let ingame_overlay = IngameOverlayView::new();
    let break_overlay = BreakOverlayView::new();

    let x = settings.taiko.hit_position_x_px();
    let y = settings.taiko.hit_position_y_perc() * graphics.height;

    let taiko_renderer = TaikoRenderer::new(
      &graphics.device,
      &graphics.queue,
      graphics.config.format,
      TaikoRendererConfig {
        width: graphics.width,
        height: graphics.height,
        scale_factor: graphics.scale_factor,
        gameplay_scale: settings.taiko.gameplay_scale(),
        conveyor_zoom: settings.taiko.conveyor_zoom(),
        hit_position_x: x,
        hit_position_y: y,
        don: settings.taiko.don_color(),
        kat: settings.taiko.kat_color(),
        // Apparently setting it to f64::INFINITY leads to a crash, see https://github.com/gfx-rs/wgpu/issues/6082
        hit_animation_height: if settings.taiko.hit_animation() { 12.5 } else { 9999999.0 },
      },
    );

    let mut sprite_renderer = SpriteRenderer::new(
      &graphics.device,
      &graphics.queue,
      graphics.config.format,
      graphics.width,
      graphics.height,
      graphics.scale_factor,
    );

    let gameplay_scale = settings.taiko.gameplay_scale() as f32;
    let x_drum = x - 160.0 * gameplay_scale;
    let hit_result_display = HitResultDisplay::new(graphics, &mut sprite_renderer, x, y, gameplay_scale);
    let hit_drum_display = HitDrumDisplay::new(graphics, &mut sprite_renderer, x_drum, y, gameplay_scale);

    let taiko_player = TaikoPlayer::new();
    let score_processor = ScoreProcessor::default();

    let don_hitsound = audio.load_sound("./assets/red.wav");
    let kat_hitsound = audio.load_sound("./assets/blue.wav");

    let taiko_circle_size = 128.0 * gameplay_scale;
    let size = Vec2::splat(taiko_circle_size);
    let origin = Origin::CenterCenter;
    let image = image::open("./assets/hit_position.png").unwrap();
    let texture = sprite_renderer.add_texture(&graphics.device, &graphics.queue, &image);
    let hit_pos_sprite =
      sprite_renderer.alloc_sprite(&graphics.device, vec2(x, y), size, origin, false, false, texture);

    return Self {
      event_bus,
      audio_controller: audio.controller(),

      taiko_renderer,
      sprite_renderer,

      hit_result_display,
      hit_drum_display,
      ingame_overlay,
      break_overlay,

      score_processor,
      taiko_player,

      don_hitsound,
      kat_hitsound,

      hit_pos_sprite,

      hit_position_x_px: x,
      hit_position_y_px: y,
      hit_position_y_perc: settings.taiko.hit_position_y_perc(),
      gameplay_scale,
    };
  }
}

impl GameplayScreen {
  pub fn hit(&mut self, input: TaikoInput, graphics: &Graphics, audio: &mut GameAudio) {
    let time = audio.position();

    match input {
      TaikoInput::DonRight | TaikoInput::DonLeft => {
        self.audio_controller.play_sound(self.don_hitsound.clone());
      }

      TaikoInput::KatLeft | TaikoInput::KatRight => {
        self.audio_controller.play_sound(self.kat_hitsound.clone());
      }
    }

    self.hit_drum_display.hit(input);

    if let Some((result, hit_idx)) = self.taiko_player.hit(time, input) {
      self.score_processor.feed(time, Some(input), result.judgement);
      self.hit_result_display.update_hit_result(graphics, &mut self.sprite_renderer, result.judgement);

      if result.judgement != Judgement::Miss {
        self.taiko_renderer.set_hit(&graphics.queue, hit_idx, time);
      }

      if result.hit_delta.abs() <= self.taiko_player.hit_window_150() {
        self.ingame_overlay.hit(result.hit_delta);
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

    self.hit_result_display.reset(graphics, &mut self.sprite_renderer);
    self.taiko_renderer.load_beatmap(&graphics.device, beatmap.clone());
    self.taiko_player.play(beatmap, beatmap_path.to_owned());
    std::mem::take(&mut self.score_processor);

    audio.set_position(Time::zero() - audio.lead_in);
    audio.set_playing(true);
  }

  pub fn reset(&mut self, graphics: &Graphics, audio: &mut GameAudio) {
    self.hit_result_display.reset(graphics, &mut self.sprite_renderer);
    self.taiko_renderer.restart_beatmap(&graphics.queue);
    self.taiko_player.reset();

    std::mem::take(&mut self.score_processor);

    audio.set_playing(false);
    audio.set_position(Time::zero() - audio.lead_in);
    audio.set_playing(true);
  }

  pub fn skip_break(&mut self, audio: &mut GameAudio, time: Time) {
    self.taiko_player.skip_break(audio, time);
  }
}

impl GameplayScreen {
  pub fn prepare(&mut self, core: &mut Core<Client>, audio: &mut GameAudio, settings: &Settings) {
    let time = audio.position();

    if self.taiko_player.has_ended(time, audio) {
      // Finish the play if beatmap is over.
      let path = self.taiko_player.beatmap_path().clone();
      let score = self.score_processor.export(Timestamp::now(), settings.profile.username().clone());
      self.event_bus.send(ClientEvent::ShowResultScreen { path, score });
    }

    self.hit_result_display.prepare(&core.graphics, &mut self.sprite_renderer);
    self.hit_drum_display.prepare(&core.graphics, &mut self.sprite_renderer);

    while self.taiko_player.process_miss(time) {
      self.score_processor.feed(time, None, Judgement::Miss);
      self
        .hit_result_display
        .update_hit_result(&core.graphics, &mut self.sprite_renderer, Judgement::Miss);
    }

    self.taiko_renderer.prepare(&core.graphics.queue, time);

    let hit_window_150 = self.taiko_player.hit_window_150();
    let hit_window_300 = self.taiko_player.hit_window_300();
    let score_processor = &self.score_processor;
    self.ingame_overlay.prepare(core, audio, score_processor, hit_window_150, hit_window_300);

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
    self.sprite_renderer.render(rpass);
    self.taiko_renderer.render(rpass);
  }
}

impl GameplayScreen {
  pub fn set_gameplay_scale(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f64) {
    self.gameplay_scale = value as f32;

    self.hit_result_display.set_gameplay_scale(value as f32);
    self.hit_drum_display.set_gameplay_scale(device, &mut self.sprite_renderer, value as f32);

    let x_drum = self.hit_position_x_px - 160.0 * self.gameplay_scale;
    self.hit_drum_display.set_hit_position_x(device, &mut self.sprite_renderer, x_drum);

    self.sprite_renderer.mutate_sprite(device, self.hit_pos_sprite, |model| {
      let circle_size = 128.0 * value as f32;
      model.scale = vec2(circle_size, circle_size);
    });

    self.taiko_renderer.set_gameplay_scale(queue, value);
  }

  pub fn set_conveyor_zoom(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f64) {
    self.taiko_renderer.set_conveyor_zoom(device, queue, value);
  }

  pub fn set_hit_position_x_px(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.hit_position_x_px = value;

    self.sprite_renderer.mutate_sprite(device, self.hit_pos_sprite, |model| {
      model.position.x = self.hit_position_x_px;
    });

    let x_drum = self.hit_position_x_px - 160.0 * self.gameplay_scale;
    self.hit_drum_display.set_hit_position_x(device, &mut self.sprite_renderer, x_drum);

    self
      .hit_result_display
      .set_hit_position_x(device, &mut self.sprite_renderer, self.hit_position_x_px);

    self.taiko_renderer.set_hit_position_x(queue, self.hit_position_x_px);
  }

  pub fn set_hit_position_y_perc(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.hit_position_y_perc = value;
    self.hit_position_y_px = self.taiko_renderer.config.height * self.hit_position_y_perc;

    self.sprite_renderer.mutate_sprite(device, self.hit_pos_sprite, |model| {
      model.position.y = self.hit_position_y_px;
    });

    self.hit_drum_display.set_hit_position_y(device, &mut self.sprite_renderer, self.hit_position_y_px);

    self
      .hit_result_display
      .set_hit_position_y(device, &mut self.sprite_renderer, self.hit_position_y_px);

    self.taiko_renderer.set_hit_position_y(queue, self.hit_position_y_px);
  }

  pub fn set_don_color(&mut self, device: &wgpu::Device, value: Color) {
    self.taiko_renderer.set_don_color(device, value);
  }

  pub fn set_kat_color(&mut self, device: &wgpu::Device, value: Color) {
    self.taiko_renderer.set_kat_color(device, value);
  }

  pub fn set_hit_animation_height(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat, value: f64) {
    self.taiko_renderer.set_hit_animation_height(device, format, value);
  }

  pub fn set_delta_bar_width(&mut self, width: f32) {
    self.ingame_overlay.delta_bar().set_bar_width(width);
  }

  pub fn set_delta_bar_height(&mut self, height: f32) {
    self.ingame_overlay.delta_bar().set_bar_height(height);
  }

  pub fn set_delta_bar_opacity(&mut self, opacity: f32) {
    self.ingame_overlay.delta_bar().set_bar_opacity(opacity);
  }

  pub fn set_delta_marker_width(&mut self, width: f32) {
    self.ingame_overlay.delta_bar().set_marker_width(width);
  }

  pub fn set_delta_marker_height(&mut self, height: f32) {
    self.ingame_overlay.delta_bar().set_marker_height(height);
  }

  pub fn set_delta_marker_opacity(&mut self, opacity: f32) {
    self.ingame_overlay.delta_bar().set_marker_opacity(opacity);
  }

  pub fn set_delta_marker_duration(&mut self, duration: Time) {
    self.ingame_overlay.delta_bar().set_marker_duration(duration);
  }

  pub fn set_delta_marker_fade(&mut self, fade: Time) {
    self.ingame_overlay.delta_bar().set_marker_fade(fade);
  }
}

impl Drawable for GameplayScreen {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.taiko_renderer.recreate(device, queue, format);
    self.sprite_renderer.recreate(device, queue, format);
  }

  fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: f32, height: f32) {
    self.taiko_renderer.resize(device, queue, width, height);
    self.sprite_renderer.resize(device, queue, width, height);
    self.set_hit_position_y_perc(device, queue, self.hit_position_y_perc);
  }

  fn resize_width(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.taiko_renderer.resize_width(device, queue, value);
    self.sprite_renderer.resize_width(device, queue, value);
    self.set_hit_position_y_perc(device, queue, self.hit_position_y_perc);
  }

  fn resize_height(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.taiko_renderer.resize_height(device, queue, value);
    self.sprite_renderer.resize_height(device, queue, value);
    self.set_hit_position_y_perc(device, queue, self.hit_position_y_perc);
  }

  fn rescale(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.taiko_renderer.rescale(device, queue, value);
    self.sprite_renderer.rescale(device, queue, value);
  }
}
