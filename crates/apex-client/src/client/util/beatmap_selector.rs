use std::sync::Arc;

use nucleo::{pattern::{CaseMatching, Normalization}, Nucleo};

use crate::{client::{event::ClientEvent, gameplay::beatmap_cache::BeatmapCache}, core::event::EventBus};

pub struct BeatmapSelector {
  matcher: Nucleo<(usize, String)>,

  search_query: String,
  selected_idx: usize,
}

impl BeatmapSelector {
  pub fn new(beatmap_cache: &BeatmapCache) -> Self {
    let matcher = Nucleo::new(
      nucleo::Config::DEFAULT,
      Arc::new(|| {}),
      std::thread::available_parallelism().map(|x| x.get()).ok(),
      1,
    );

    for (i, (_, info)) in beatmap_cache.iter().enumerate() {
      let q_str = format!("{}{}{}{}", &info.title, &info.artist, &info.difficulty, &info.creator);
      matcher.injector().push((i, q_str), |(_, q_str), cols| {
        cols[0] = q_str.clone().into();
      });
    }

    return Self {
      matcher,
      search_query: String::new(),
      selected_idx: 0,
    }
  }

  pub fn tick(&mut self) {
    self.matcher.tick(10);
  }

  pub fn selected(&self) -> usize {
    return self.selected_idx;
  }

  pub fn set_selected(&mut self, idx: usize) {
    self.selected_idx = idx;
  }

  pub fn matched(&self) -> impl Iterator<Item = usize> + '_ {
    return self.matcher.snapshot().matched_items(..).map(|x| x.data.0);
  }

  pub fn query(&self) -> &str {
    return &self.search_query;
  }

  pub fn query_mut(&mut self) -> &mut String {
    return &mut self.search_query;
  }

  pub fn clear_query(&mut self) {
    self.search_query.clear();
    self.matcher.pattern.reparse(0, &self.search_query, CaseMatching::Ignore, Normalization::Smart, false);
  }

  pub fn push_query(&mut self, c: char) {
    self.search_query.push(c);
    self.matcher.pattern.reparse(0, &self.search_query, CaseMatching::Ignore, Normalization::Smart, false);
  }

  pub fn pop_query(&mut self) {
    self.search_query.pop();
    self.matcher.pattern.reparse(0, &self.search_query, CaseMatching::Ignore, Normalization::Smart, false);
  }

  pub fn has_query(&self) -> bool {
    return !self.search_query.is_empty();
  }

  pub fn select_next(&mut self) {
    let snapshot = self.matcher.snapshot();
    let mut iter = snapshot.matched_items(..);
    let Some(idx) = iter.position(|x| x.data.0 == self.selected_idx) else {
      self.selected_idx = snapshot.matched_items(..).next().map(|x| x.data.0).unwrap_or(0);
      return;
    };

    if let Some(info) = snapshot.get_matched_item(idx as u32 + 1) {
      self.selected_idx = info.data.0;
    }
  }

  pub fn select_prev(&mut self) {
    let snapshot = self.matcher.snapshot();
    let mut iter = snapshot.matched_items(..);
    let Some(idx) = iter.position(|x| x.data.0 == self.selected_idx) else {
      self.selected_idx = snapshot.matched_items(..).next().map(|x| x.data.0).unwrap_or(0);
      return;
    };

    if let Some(info) = snapshot.get_matched_item(idx as u32 - 1) {
      self.selected_idx = info.data.0;
    }
  }

  pub fn select(&self, event_bus: &EventBus<ClientEvent>, beatmap_cache: &BeatmapCache) -> Result<(), ()> {
    let Some((path, _)) = beatmap_cache.get_index(self.selected_idx) else { return Err(()) };
    event_bus.send(ClientEvent::SelectBeatmap { path: path.clone() });

    return Ok(());
  }
}
