use super::{
  gameplay::beatmap::BeatmapHash,
  score::{score::Score, score_cache::ScoreId},
};

#[derive(Debug)]
pub enum ClientEvent {
  RetryBeatmap,
  ToggleSettings,
  ToggleRecordingWindow,
  ShowResultScreen {
    beatmap_hash: BeatmapHash,
    score: Score,
  },
  ViewScore {
    beatmap_hash: BeatmapHash,
    score_id: ScoreId,
  },
  PickBeatmap {
    beatmap_hash: BeatmapHash,
  },
  SelectBeatmap,
}
