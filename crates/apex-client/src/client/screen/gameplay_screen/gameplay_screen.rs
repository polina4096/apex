use std::{fs::File, io::{BufReader, Cursor}, path::Path};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source as _};

use crate::{client::{client::Client, graphics::taiko_renderer::taiko_renderer::TaikoRenderer, gui::ingame_overlay::ingame_overlay_view::IngameOverlayView, taiko::beatmap::Beatmap}, core::{core::Core, graphics::graphics::Graphics, time::{clock::{AbstractClock, Clock}, time::Time}}};

use super::gameplay_playback_controller::GameplayPlaybackController;

#[derive(Debug, Clone, Copy)]
pub enum TaikoInput {
  KatOne,
  DonOne,
  KatTwo,
  DonTwo,
}

pub struct GameplayScreen {
  taiko_renderer: TaikoRenderer,
  ingame_overlay: IngameOverlayView,

  stream: OutputStream,
  stream_handle: OutputStreamHandle,
  sink: Sink,

  beatmap: Option<Beatmap>,
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
      beatmap,
    };
  }

  pub fn hit(&mut self, graphics: &Graphics, input: TaikoInput) {
    let hit_time = self.clock.position();
    let Some(beatmap) = &self.beatmap else { return };

    self.ingame_overlay.hit(input);
    self.taiko_renderer.hit(graphics, beatmap, hit_time, input);
  }

  pub fn play(&mut self, beatmap_path: &Path, graphics: &Graphics) {
    let data = std::fs::read_to_string(beatmap_path).unwrap();
    let beatmap = Beatmap::from(data);
    let end_time = beatmap.hit_objects.last().unwrap().time;

    self.taiko_renderer.prepare_instances(graphics, &beatmap);
    self.taiko_renderer.culling = 0;
    self.taiko_renderer.hit_idx = 0;

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
    let Some(beatmap) = &self.beatmap else { return };

    self.taiko_renderer.prepare(&core.graphics, beatmap, &mut self.clock);

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
