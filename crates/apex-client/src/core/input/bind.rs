use std::{fmt::Display, hash::Hash};

use log::warn;
use fxhash::FxHashMap;
use winit::keyboard::{ModifiersState, PhysicalKey};

/// Container providing ergonomic API to store and access keybinds
pub struct KeybindManager<T> {
  /// Maps key combinations to the respective binds values
  binds: FxHashMap<KeyCombination, Bind<T>>,
}

impl<T> Default for KeybindManager<T> {
  fn default() -> Self {
    Self {
      binds: Default::default(),
    }
  }
}

impl<T: Copy + Eq + Hash> KeybindManager<T> {
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

  pub fn as_vec(&self) -> Vec<(KeyCombination, Bind<T>)> {
    let mut cache: Vec<_> = self.binds.iter().map(|x|(*x.0, x.1.clone())).collect();
    cache.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    return cache;
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
    } else { return }
  }
}

/// Represents a key combination: <A>, <Ctrl + A>, <Ctrl + Shift + D>
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct KeyCombination {
    pub key       : PhysicalKey,
    pub modifiers : ModifiersState,
}

impl KeyCombination {
  pub fn new(key: PhysicalKey, modifiers: ModifiersState) -> Self {
    return Self { key, modifiers };
  }
}

impl Display for KeyCombination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      if self.modifiers.control_key() { write!(f,  "Ctrl + ")?; }
      if self.modifiers.shift_key()   { write!(f, "Shift + ")?; }
      if self.modifiers.alt_key()     { write!(f,   "Alt + ")?; }

      return match self.key {
        PhysicalKey::Code(key)         => write!(f, "{:?}", key),
        PhysicalKey::Unidentified(key) => write!(f, "{:?}", key),
    }
  }
}

/// Action which is usually dispatched by the input system, or rendered in the UI
#[derive(Clone)]
pub struct Bind<T> {
    pub id          : T,
    pub name        : String,
    pub description : String,
}
