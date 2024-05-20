use std::{fs::File, io::BufReader, path::Path};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source as _};

use crate::{client::{client::Client, gameplay::{beatmap::Beatmap, taiko_player::{TaikoPlayer, TaikoPlayerInput}}, graphics::taiko_renderer::taiko_renderer::TaikoRenderer, ui::ingame_overlay::ingame_overlay_view::IngameOverlayView}, core::{core::Core, graphics::graphics::Graphics, time::{clock::{AbstractClock, Clock}, time::Time}}};

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
      beatmap,
    };
  }

  pub fn hit(&mut self, graphics: &Graphics, input: TaikoPlayerInput) {
    let hit_time = self.clock.position();
    let Some(beatmap) = &self.beatmap else { return };

    self.ingame_overlay.hit(input);
    self.player.hit(hit_time, input, beatmap, |result, idx| {
      self.taiko_renderer.hit(graphics, hit_time, idx);
      self.ingame_overlay.show_hit_result(result);
    });
  }

  pub fn reset(&mut self, graphics: &Graphics) {
    self.taiko_renderer.reset_instances(graphics);
    self.taiko_renderer.culling = 0;
    self.player.reset();

    self.clock.set_playing(false);
    self.sink.pause();

    self.clock.set_position(Time::zero());
    self.sink.try_seek(std::time::Duration::ZERO).unwrap();

    self.clock.set_playing(true);
    self.sink.play();
  }

  pub fn play(&mut self, beatmap_path: &Path, graphics: &Graphics) {
    let data = std::fs::read_to_string(beatmap_path).unwrap();
    let beatmap = Beatmap::from(data);
    let end_time = beatmap.hit_objects.last().unwrap().time;

    self.taiko_renderer.prepare_instances(graphics, &beatmap);
    self.taiko_renderer.culling = 0;

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

  pub fn prepare(&mut self, core: &mut Core<Client>) {
    self.taiko_renderer.prepare(&core.graphics, &mut self.clock);
    self.ingame_overlay.prepare(core, GameplayPlaybackController {
      taiko_renderer: &mut self.taiko_renderer,
      clock: &mut self.clock,
      sink: &mut self.sink,
    });
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
}
