use std::{fs::File, io::BufReader, path::Path};

use rodio::{Decoder, Source as _};
use tap::Tap;

use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    gameplay::{
      beatmap::Beatmap,
      score_processor::{ScoreProcessor, ScoreProcessorEvent},
      taiko_player::{TaikoPlayer, TaikoPlayerInput},
    },
    graphics::taiko_renderer::taiko_renderer::TaikoRenderer,
    state::AppState,
    ui::ingame_overlay::{HitResult, IngameOverlayView},
    util::beatmap_audio::BeatmapAudio,
  },
  core::{
    core::Core,
    event::EventBus,
    graphics::{drawable::Drawable, graphics::Graphics},
    time::{clock::AbstractClock, time::Time},
  },
};

pub struct GameplayScreen {
  event_bus: EventBus<ClientEvent>,

  taiko_renderer: TaikoRenderer,
  ingame_overlay: IngameOverlayView,

  beatmap: Option<Beatmap>,
  score: ScoreProcessor,
  player: TaikoPlayer,
  audio: BeatmapAudio,
}

impl GameplayScreen {
  pub fn new(event_bus: EventBus<ClientEvent>, graphics: &Graphics) -> Self {
    let taiko_renderer = TaikoRenderer::new(graphics);
    let ingame_overlay = IngameOverlayView::new();

    let beatmap = None;
    let score = ScoreProcessor::default();
    let player = TaikoPlayer::new();
    let audio = BeatmapAudio::new().tap_mut(|x| {
      x.lead_in = Time::from_ms(1000);
      x.lead_out = Time::from_ms(1000);
    });

    return Self {
      event_bus,

      taiko_renderer,
      ingame_overlay,

      beatmap,
      score,
      player,
      audio,
    };
  }

  pub fn hit(&mut self, input: TaikoPlayerInput, graphics: &Graphics, state: &AppState) {
    let Some(beatmap) = &self.beatmap else { return };
    let time = self.audio.position();

    self.ingame_overlay.hit(input);

    self.player.hit(time, input, beatmap, |result, idx| {
      self.score.feed(ScoreProcessorEvent { result });
      self.taiko_renderer.set_hit(graphics, time, idx, state);
      self.ingame_overlay.show_hit_result(result);
    });
  }

  pub fn reset(&mut self, graphics: &Graphics) {
    self.taiko_renderer.reset_instances(graphics);
    self.player.reset();

    std::mem::take(&mut self.score);

    self.audio.set_playing(false);
    self.audio.set_position(Time::zero());
    self.audio.set_playing(true);
  }

  pub fn play(&mut self, beatmap_path: &Path, graphics: &Graphics, state: &AppState) {
    let data = std::fs::read_to_string(beatmap_path).unwrap();
    let beatmap = Beatmap::from(data);
    let end_time = beatmap.hit_objects.last().unwrap().time;

    self.taiko_renderer.prepare_instances(graphics, &beatmap, state);
    std::mem::take(&mut self.score);
    self.player.reset();

    let audio_path = beatmap_path.parent().unwrap().join(&beatmap.audio);
    let file = BufReader::new(File::open(audio_path).unwrap());
    let source = Decoder::new(file).unwrap();

    self.audio.set_source(source.convert_samples::<f32>());
    self.audio.set_length(end_time);

    self.beatmap = Some(beatmap);

    self.audio.set_position(Time::zero());
    self.audio.set_playing(true);
    // self.sink.play();
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, state: &AppState) {
    let time = self.audio.position();

    if self.audio.lead_in.to_ms() as i32 != state.gameplay.lead_in {
      self.audio.lead_in = Time::from_ms(state.gameplay.lead_in);
    }

    if self.audio.lead_out.to_ms() as i32 != state.gameplay.lead_out {
      self.audio.lead_out = Time::from_ms(state.gameplay.lead_out);
    }

    // delay after the last hit object before result screen
    if time >= self.audio.length() + self.audio.lead_out {
      self.event_bus.send(ClientEvent::ShowResultScreen);
    }

    if let Some(beatmap) = &self.beatmap {
      self.player.tick(time, beatmap, |_idx| {
        self.score.feed(ScoreProcessorEvent { result: HitResult::Miss });
        self.ingame_overlay.show_hit_result(HitResult::Miss);
      });
    }

    self.taiko_renderer.prepare(&core.graphics, time, state);
    self.ingame_overlay.prepare(core, &mut self.audio, &self.score, state);
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

  pub fn rebuild_instances(&mut self, graphics: &Graphics, state: &AppState) {
    if let Some(beatmap) = &self.beatmap {
      self.taiko_renderer.prepare_instances(graphics, beatmap, state);
    }
  }
}

impl Drawable for GameplayScreen {
  fn recreate(&mut self, graphics: &Graphics) {
    self.taiko_renderer.recreate(graphics);
  }
}
