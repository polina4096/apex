use std::{fs::File, io::BufReader, path::Path};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source as _};

use crate::{client::{client::Client, gameplay::{beatmap::Beatmap, score_processor::{ScoreProcessor, ScoreProcessorEvent}, taiko_player::{TaikoPlayer, TaikoPlayerInput}}, graphics::taiko_renderer::taiko_renderer::TaikoRenderer, state::GameState, ui::ingame_overlay::{HitResult, IngameOverlayView}}, core::{core::Core, graphics::graphics::Graphics, time::{clock::{AbstractClock, Clock}, time::Time}}};

use super::gameplay_playback_controller::GameplayPlaybackController;

pub struct GameplayScreen {
  taiko_renderer: TaikoRenderer,
  ingame_overlay: IngameOverlayView,

  #[allow(unused)]
  stream: OutputStream,
  #[allow(unused)]
  stream_handle: OutputStreamHandle,
  sink: Sink,

  beatmap: Option<Beatmap>,
  player: TaikoPlayer,
  score: ScoreProcessor,
  clock: Clock,
}

impl GameplayScreen {
  pub fn playback_controller(&mut self) -> GameplayPlaybackController {
    return GameplayPlaybackController {
      taiko_renderer: &mut self.taiko_renderer,
      clock: &mut self.clock,
      sink: &mut self.sink,
    };
  }
}

impl GameplayScreen {
  pub fn new(graphics: &Graphics) -> Self {
    let taiko_renderer = TaikoRenderer::new(graphics);
    let ingame_overlay = IngameOverlayView::new();

    let beatmap = None;
    let player = TaikoPlayer::new();
    let score = ScoreProcessor::default();
    let clock = Clock::new();

    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    return Self {
      taiko_renderer,
      ingame_overlay,

      stream,
      stream_handle,
      sink,

      clock,
      player,
      score,
      beatmap,
    };
  }

  pub fn hit(&mut self, input: TaikoPlayerInput, graphics: &Graphics, state: &GameState) {
    let Some(beatmap) = &self.beatmap else { return };
    let offset = Time::from_ms(state.gameplay.audio_offset);
    let time = self.clock.position() + offset;

    self.ingame_overlay.hit(input);

    self.player.hit(time, input, beatmap, |result, idx| {
      self.score.feed(ScoreProcessorEvent { result });
      self.taiko_renderer.set_hit(graphics, time, idx, state);
      self.ingame_overlay.show_hit_result(result);
    });
  }

  pub fn reset(&mut self, graphics: &Graphics) {
    self.taiko_renderer.reset_instances(graphics);
    self.taiko_renderer.culling = 0;
    std::mem::take(&mut self.score);
    self.player.reset();

    self.clock.set_playing(false);
    self.sink.pause();

    self.clock.set_position(Time::zero());
    self.sink.try_seek(std::time::Duration::ZERO).unwrap();

    self.clock.set_playing(true);
    self.sink.play();
  }

  pub fn play(&mut self, beatmap_path: &Path, graphics: &Graphics, state: &GameState) {
    let data = std::fs::read_to_string(beatmap_path).unwrap();
    let beatmap = Beatmap::from(data);
    let end_time = beatmap.hit_objects.last().unwrap().time;

    self.taiko_renderer.prepare_instances(graphics, &beatmap, state);
    self.taiko_renderer.culling = 0;
    std::mem::take(&mut self.score);
    self.player.reset();

    let audio_path = beatmap_path.parent().unwrap().join(&beatmap.audio);
    let file = BufReader::new(File::open(audio_path).unwrap());
    let source = Decoder::new(file).unwrap();
    self.sink.clear();
    self.sink.append(source.convert_samples::<f32>());

    self.beatmap = Some(beatmap);

    self.clock.set_length(end_time);
    self.clock.set_position(Time::zero());
    self.clock.set_playing(true);
    self.sink.play();
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, state: &GameState) {
    let offset = Time::from_ms(state.gameplay.audio_offset);
    let time = self.clock.position() + offset;

    if let Some(beatmap) = &self.beatmap {
      self.player.tick(time, beatmap, |_idx| {
        self.score.feed(ScoreProcessorEvent { result: HitResult::Miss });
        self.ingame_overlay.show_hit_result(HitResult::Miss);
      });
    }

    self.taiko_renderer.prepare(&core.graphics, time, state);
    self.ingame_overlay.prepare(core, GameplayPlaybackController {
      taiko_renderer: &mut self.taiko_renderer,
      clock: &mut self.clock,
      sink: &mut self.sink,
    }, &self.score, state);
  }

  pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
    self.taiko_renderer.render(rpass);
  }

  pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
    self.taiko_renderer.scene.resize(size);
  }

  pub fn scale(&mut self, scale_factor: f64) {
    self.taiko_renderer.scene.scale(scale_factor);
  }

  pub fn rebuild_instances(&mut self, graphics: &Graphics, state: &GameState) {
    if let Some(beatmap) = &self.beatmap {
      self.taiko_renderer.prepare_instances(graphics, beatmap, state);
    }
  }
}
