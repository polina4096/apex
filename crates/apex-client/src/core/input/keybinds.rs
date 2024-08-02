use std::{fmt::Display, hash::Hash, marker::PhantomData};

use ahash::AHashMap;
use log::warn;
use serde::{
  de::{DeserializeOwned, MapAccess, Visitor},
  ser::SerializeMap,
  Deserialize, Deserializer, Serialize,
};
use tap::Tap;
use winit::keyboard::{ModifiersState, PhysicalKey};

use crate::core::data::persistent::Persistent;

use super::action::AppActions;

/// Container providing ergonomic API to store and access keybinds
pub struct Keybinds<T> {
  /// Maps key combinations to the respective binds values
  binds: AHashMap<KeyCombination, Bind<T>>,
}

impl<T> Default for Keybinds<T> {
  fn default() -> Self {
    Self { binds: Default::default() }
  }
}

impl<T> Keybinds<T> {
  pub fn merge(&mut self, other: Self) {
    for (key, bind) in other.binds {
      self.binds.insert(key, bind);
    }
  }

  pub fn add(&mut self, key: KeyCombination, bind: Bind<T>) {
    self.binds.insert(key, bind);
  }

  pub fn remove(&mut self, key: &KeyCombination) -> Option<Bind<T>> {
    let bind = self.binds.remove(key);
    return bind;
  }

  pub fn get(&mut self, key: &KeyCombination) -> Option<&Bind<T>> {
    return self.binds.get(key);
  }

  pub fn len(&self) -> usize {
    return self.binds.len();
  }

  pub fn is_empty(&self) -> bool {
    return self.binds.is_empty();
  }

  pub fn rebind(&mut self, old_key: KeyCombination, new_key: KeyCombination) {
    if old_key != new_key {
      #[allow(clippy::collapsible_else_if)]
      if self.binds.contains_key(&new_key) {
        if let Some([a, b]) = self.binds.get_many_mut([&new_key, &old_key]) {
          std::mem::swap(a, b);
          return;
        }
      } else {
        if let Some(keybind) = self.binds.remove(&old_key) {
          self.binds.insert(new_key, keybind);
          return;
        }
      }

      warn!("Failed to find key: {}", old_key);
    } else {
      return;
    }
  }
}

impl<T: Copy + Eq + Hash> Keybinds<T> {
  pub fn as_vec(&self) -> Vec<(KeyCombination, Bind<T>)> {
    let mut cache: Vec<_> = self.binds.iter().map(|x| (*x.0, x.1.clone())).collect();
    cache.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    return cache;
  }
}

impl<'de, T: AppActions + Serialize + Deserialize<'de>> Serialize for Keybinds<T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut state = serializer.serialize_map(Some(self.binds.len()))?;

    for (key, bind) in self.binds.iter() {
      state.serialize_entry(&bind.id, key)?;
    }

    return state.end();
  }
}

struct KeybindsVisitor<T>(PhantomData<T>);

impl<'de, T: AppActions + Copy + Eq + Hash + Deserialize<'de>> Visitor<'de> for KeybindsVisitor<T> {
  type Value = Keybinds<T>;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("an integer between -2^31 and 2^31")
  }

  fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut keybinds = Keybinds::default();

    while let Some((id, key)) = access.next_entry()? {
      let (name, description) = T::action_info(&id);

      keybinds.add(key, Bind { id, name, description });
    }

    return Ok(keybinds);
  }
}

impl<'de, T: AppActions + Copy + Eq + Hash + Deserialize<'de>> Deserialize<'de> for Keybinds<T> {
  fn deserialize<D>(deserializer: D) -> Result<Keybinds<T>, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_map(KeybindsVisitor::<T>(PhantomData))
  }
}

impl<T: AppActions + Copy + Eq + Hash + Serialize + DeserializeOwned> Persistent for Keybinds<T> {
  fn load(path: impl AsRef<std::path::Path>) -> Self {
    let absolute = path.as_ref().canonicalize().unwrap_or(path.as_ref().to_owned());
    log::info!("Loading settings from `{}`", absolute.display());

    return std::fs::read_to_string(&path)
      .map(|data| {
        return toml::from_str(&data).unwrap_or_else(|e| {
          log::error!("Failed to parse config file, falling back to default config: {}", e);
          return Self::default().tap_mut(T::insert_keybinds);
        });
      })
      .unwrap_or_else(|e| {
        let default = Self::default().tap_mut(T::insert_keybinds);

        match e.kind() {
          std::io::ErrorKind::NotFound => {
            log::warn!("Failed to open config file, file not found. Creating a default config file...");
            let default_data = toml::to_string_pretty(&default).expect("Failed to serialize default config");
            if let Err(e) = std::fs::write(&path, default_data) {
              log::error!("Failed to write default config file: {}", e);
            }
          }

          std::io::ErrorKind::PermissionDenied => {
            log::warn!("Failed to open config file, insufficient permissions. Falling back to default configuration.");
          }

          _ => {
            log::error!("Failed to access config file: {}. Falling back to default configuration.", e);
          }
        }

        return default;
      });
  }

  fn save(&self, path: impl AsRef<std::path::Path>) {
    let data = match toml::to_string_pretty(&self) {
      Ok(data) => data,
      Err(e) => {
        log::error!("Failed to serialize settings: {}", e);
        return;
      }
    };

    if let Err(e) = std::fs::write(&path, data) {
      log::error!("Failed to write settings to file: {}", e);
      return;
    }

    let path = path.as_ref().canonicalize().unwrap_or(path.as_ref().to_owned());
    log::info!("Settings successfully written to `{}`", path.display());
  }
}

/// Represents a key combination: <A>, <Ctrl + A>, <Ctrl + Shift + D>
#[derive(Debug, Serialize, Deserialize, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct KeyCombination {
  pub key: PhysicalKey,
  pub modifiers: ModifiersState,
}

impl KeyCombination {
  pub fn new(key: PhysicalKey, modifiers: ModifiersState) -> Self {
    return Self { key, modifiers };
  }
}

impl Display for KeyCombination {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    #[rustfmt::skip] {
      if self.modifiers.control_key() { write!(f,  "Ctrl + ")?; }
      if self.modifiers.shift_key()   { write!(f, "Shift + ")?; }
      if self.modifiers.alt_key()     { write!(f,   "Alt + ")?; }
      if self.modifiers.super_key()   { write!(f, "Super + ")?; }
    };

    return match self.key {
      PhysicalKey::Code(key) => write!(f, "{:?}", key),
      PhysicalKey::Unidentified(key) => write!(f, "{:?}", key),
    };
  }
}

#[macro_export]
macro_rules! key_comb {
  (Ctrl + $key:ident) => {
    KeyCombination::new(PhysicalKey::Code(KeyCode::$key), ModifiersState::CONTROL)
  };

  (Shift + $key:ident) => {
    KeyCombination::new(PhysicalKey::Code(KeyCode::$key), ModifiersState::SHIFT)
  };

  (Alt + $key:ident) => {
    KeyCombination::new(PhysicalKey::Code(KeyCode::$key), ModifiersState::ALT)
  };

  (Super + $key:ident) => {
    KeyCombination::new(PhysicalKey::Code(KeyCode::$key), ModifiersState::SUPER)
  };

  ($key:ident) => {
    KeyCombination::new(PhysicalKey::Code(KeyCode::$key), ModifiersState::empty())
  };
}

/// Action which is usually dispatched by the input system, or rendered in the UI
#[rustfmt::skip]
#[derive(Clone)]
pub struct Bind<T> {
  pub id          : T,
  pub name        : String,
  pub description : String,
}
