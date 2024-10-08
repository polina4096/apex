use std::sync::Arc;

use instant::Instant;
use log::debug;
use nucleo::{
  pattern::{CaseMatching, Normalization},
  Nucleo,
};

use crate::client::gameplay::beatmap_cache::BeatmapCache;

pub struct BeatmapSelector {
  matcher: Nucleo<(usize, String)>,

  selected_idx: usize,
  search_query: String,

  last_update: Instant,
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
      let q_str = format!("{}{}{}{}", &info.title, &info.artist, &info.variant, &info.creator);
      matcher.injector().push((i, q_str), |(_, q_str), cols| {
        cols[0] = q_str.clone().into();
      });
    }

    return Self {
      matcher,
      selected_idx: 0,
      search_query: String::new(),
      last_update: Instant::now(),
    };
  }

  pub fn tick(&mut self, beatmap_cache: &BeatmapCache) {
    self.matcher.tick(10);

    // TODO: this is going to be very slow on a large number of beatmaps, probably go with event based approach
    if beatmap_cache.last_update() > self.last_update {
      debug!("Updating beatmap list");

      self.last_update = Instant::now();
      self.matcher.restart(true);

      for (i, (_, info)) in beatmap_cache.iter().enumerate() {
        let q_str = format!("{}{}{}{}", &info.title, &info.artist, &info.variant, &info.creator);
        self.matcher.injector().push((i, q_str), |(_, q_str), cols| {
          cols[0] = q_str.clone().into();
        });
      }
    }
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

    let matcher = &mut self.matcher;
    matcher.pattern.reparse(0, &self.search_query, CaseMatching::Ignore, Normalization::Smart, false);
  }

  pub fn push_query(&mut self, c: char) {
    self.search_query.push(c);

    let matcher = &mut self.matcher;
    matcher.pattern.reparse(0, &self.search_query, CaseMatching::Ignore, Normalization::Smart, false);
  }

  pub fn pop_query(&mut self) {
    self.search_query.pop();

    let matcher = &mut self.matcher;
    matcher.pattern.reparse(0, &self.search_query, CaseMatching::Ignore, Normalization::Smart, false);
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
}
