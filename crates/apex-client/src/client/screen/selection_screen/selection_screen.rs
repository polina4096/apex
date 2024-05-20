use crate::{client::{client::Client, event::ClientEvent, ui::beatmap_list::beatmap_list_view::BeatmapListView, gameplay::beatmap_cache::BeatmapCache}, core::{core::Core, event::EventBus}};

pub struct SelectionScreen {
  beatmap_list: BeatmapListView,
}

impl SelectionScreen {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cache: &BeatmapCache) -> Self {
    let beatmap_list = BeatmapListView::new(event_bus, beatmap_cache);

    return Self {
      beatmap_list,
    };
  }

  pub fn clear_search_query(&mut self) {
    self.beatmap_list.clear_search_query();
  }

  pub fn has_search_query(&self) -> bool {
    return self.beatmap_list.has_search_query();
  }

  pub fn append_search_query(&mut self, c: char) {
    self.beatmap_list.append_search_query(c);
  }

  pub fn pop_search_query(&mut self) {
    self.beatmap_list.pop_search_query();
  }

  pub fn prepare(&mut self, core: &mut Core<Client>) {
    self.beatmap_list.prepare(core);
  }

  pub fn select_next(&mut self) {
    self.beatmap_list.select_next();
  }

  pub fn select_prev(&mut self) {
    self.beatmap_list.select_prev();
  }

  pub fn select(&mut self) {
    self.beatmap_list.select();
  }
}
