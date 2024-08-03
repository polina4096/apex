use std::path::PathBuf;

use ahash::AHashMap;
use jiff::Timestamp;
use rusqlite::Connection;

use super::{grades::Grade, score::Score};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScoreId(usize);

impl Default for ScoreId {
  fn default() -> Self {
    return Self(0);
  }
}

pub struct ScoreCache {
  conn: Connection,
  cache: AHashMap<PathBuf, Vec<ScoreId>>,
  scores: Vec<Score>,
}

impl ScoreCache {
  pub fn new(connection: Connection) -> Self {
    connection
      .execute(
        "create table if not exists scores (
           id integer primary key,
           path text not null,
           date integer not null,
           username text not null,
           score_points integer not null,
           result_300 integer not null,
           result_150 integer not null,
           result_miss integer not null,
           last_combo integer not null,
           max_combo integer not null,
           accuracy real not null,
           grade integer not null
       )",
        (),
      )
      .unwrap();

    return Self {
      conn: connection,
      cache: AHashMap::new(),
      scores: vec![Score::default()],
    };
  }

  pub fn beatmap_scores(&mut self, path: &PathBuf) -> Option<&[ScoreId]> {
    // polonius when
    if self.cache.get(path).is_some() {
      return self.cache.get(path).map(|x| x.as_slice());
    }

    let mut stmt = self.conn.prepare(
      "select date, username, score_points, result_300, result_150, result_miss, last_combo, max_combo, accuracy, grade
       from scores
       where path = ?1"
      ).unwrap();

    let scores = stmt
      .query_map((path.to_str().unwrap(),), |row| {
        return Ok(Score {
          date: Timestamp::from_millisecond(row.get(0).unwrap()).unwrap(),
          username: row.get(1).unwrap(),
          score_points: row.get::<_, i64>(2).unwrap() as usize,
          result_300: row.get::<_, i64>(3).unwrap() as usize,
          result_150: row.get::<_, i64>(4).unwrap() as usize,
          result_miss: row.get::<_, i64>(5).unwrap() as usize,
          last_combo: row.get::<_, i64>(6).unwrap() as usize,
          max_combo: row.get::<_, i64>(7).unwrap() as usize,
          accuracy: row.get::<_, f32>(8).unwrap(),
          grade: Grade::from(row.get::<_, i64>(9).unwrap() as u8),
        });
      })
      .unwrap();

    let cache = self.cache.entry(path.clone()).or_default();

    for score in scores {
      let score = score.unwrap();
      let id = ScoreId(self.scores.len());
      self.scores.push(score);
      cache.push(id);
    }

    return Some(cache.as_slice());
  }

  pub fn score_details(&self, id: ScoreId) -> &Score {
    return &self.scores[id.0];
  }

  pub fn insert(&mut self, path: PathBuf, score: Score) -> ScoreId {
    let id = ScoreId(self.scores.len());

    self.conn.execute(
      "insert into scores (id, path, date, username, score_points, result_300, result_150, result_miss, last_combo, max_combo, accuracy, grade)
       values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
       (
          id.0 as i64,
          path.to_str().unwrap(),
          score.date().as_millisecond(),
          score.username(),
          score.score_points() as i64,
          score.result_300s() as i64,
          score.result_150s() as i64,
          score.result_misses() as i64,
          score.last_combo() as i64,
          score.max_combo() as i64,
          score.accuracy() as f64,
          score.grade() as i64,
       )
    ).unwrap();

    self.scores.push(score);
    self.cache.entry(path).or_default().push(id);

    return id;
  }
}
