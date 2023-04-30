use std::{fmt::Display, collections::HashMap, hash::Hash, slice};

use ahash::RandomState;
use log::warn;
use winit::event::{VirtualKeyCode, ModifiersState};

pub struct Keybinds<T: Copy + Eq + Hash> {
    sorted : Vec<(KeyCombination, Bind<T>)>,
    binds  : HashMap<KeyCombination, Bind<T>, RandomState>,
}

impl<T: Copy + Eq + Hash> Default for Keybinds<T> {
    fn default() -> Self {
        Self {
            sorted : Default::default(),
            binds  : Default::default()
        }
    }
}

impl<T: Copy + Eq + Hash> Keybinds<T> {
    pub fn add(&mut self, key: KeyCombination, bind: Bind<T>) {
        self.binds.insert(key, bind);
        self.rebuild_sorted();
    }

    pub fn remove(&mut self, key: &KeyCombination) -> Option<Bind<T>> {
        let bind = self.binds.remove(key);
        self.rebuild_sorted();

        return bind;
    }

    pub fn get(&mut self, key: &KeyCombination) -> Option<&Bind<T>> {
        return self.binds.get(key);
    }

    pub fn iter(&self) -> slice::Iter<'_, (KeyCombination, Bind<T>)> {
        return self.sorted.iter();
    }

    pub fn rebind(&mut self, old_key: KeyCombination, new_key: KeyCombination) {
        if old_key != new_key {
            #[allow(clippy::collapsible_else_if)]
            if self.binds.contains_key(&new_key) {
                if let Some([a, b]) = self.binds.get_many_mut([&new_key, &old_key]) {
                    std::mem::swap(a, b);
                    self.rebuild_sorted();
                    return;
                }
            } else {
                if let Some(keybind) = self.binds.remove(&old_key) {
                    self.binds.insert(new_key, keybind);
                    self.rebuild_sorted();
                    return;
                }
            }

            warn!("Failed to find to rebind keys: {}", old_key);
        } else { return }
    }

    pub fn rebuild_sorted(&mut self) {
        self.sorted = self.binds.iter().map(|x|(*x.0, x.1.clone())).collect();
        self.sorted.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    } 
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct KeyCode(VirtualKeyCode);

impl Default for KeyCode {
    fn default() -> Self {
        return Self(VirtualKeyCode::Unlabeled);
    }
}

impl From<VirtualKeyCode> for KeyCode {
    fn from(value: VirtualKeyCode) -> Self {
        return Self(value);
    }
}

impl Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            VirtualKeyCode::Key1 => write!(f, "1"),
            VirtualKeyCode::Key2 => write!(f, "2"),
            VirtualKeyCode::Key3 => write!(f, "3"),
            VirtualKeyCode::Key4 => write!(f, "4"),
            VirtualKeyCode::Key5 => write!(f, "5"),
            VirtualKeyCode::Key6 => write!(f, "6"),
            VirtualKeyCode::Key7 => write!(f, "7"),
            VirtualKeyCode::Key8 => write!(f, "8"),
            VirtualKeyCode::Key9 => write!(f, "9"),
            VirtualKeyCode::Key0 => write!(f, "0"),
            VirtualKeyCode::A => write!(f, "A"),
            VirtualKeyCode::B => write!(f, "B"),
            VirtualKeyCode::C => write!(f, "C"),
            VirtualKeyCode::D => write!(f, "D"),
            VirtualKeyCode::E => write!(f, "E"),
            VirtualKeyCode::F => write!(f, "F"),
            VirtualKeyCode::G => write!(f, "G"),
            VirtualKeyCode::H => write!(f, "H"),
            VirtualKeyCode::I => write!(f, "I"),
            VirtualKeyCode::J => write!(f, "J"),
            VirtualKeyCode::K => write!(f, "K"),
            VirtualKeyCode::L => write!(f, "L"),
            VirtualKeyCode::M => write!(f, "M"),
            VirtualKeyCode::N => write!(f, "N"),
            VirtualKeyCode::O => write!(f, "O"),
            VirtualKeyCode::P => write!(f, "P"),
            VirtualKeyCode::Q => write!(f, "Q"),
            VirtualKeyCode::R => write!(f, "R"),
            VirtualKeyCode::S => write!(f, "S"),
            VirtualKeyCode::T => write!(f, "T"),
            VirtualKeyCode::U => write!(f, "U"),
            VirtualKeyCode::V => write!(f, "V"),
            VirtualKeyCode::W => write!(f, "W"),
            VirtualKeyCode::X => write!(f, "X"),
            VirtualKeyCode::Y => write!(f, "Y"),
            VirtualKeyCode::Z => write!(f, "Z"),
            VirtualKeyCode::Escape => write!(f, "Esc"),
            VirtualKeyCode::F1 => write!(f, "F1"),
            VirtualKeyCode::F2 => write!(f, "F2"),
            VirtualKeyCode::F3 => write!(f, "F3"),
            VirtualKeyCode::F4 => write!(f, "F4"),
            VirtualKeyCode::F5 => write!(f, "F5"),
            VirtualKeyCode::F6 => write!(f, "F6"),
            VirtualKeyCode::F7 => write!(f, "F7"),
            VirtualKeyCode::F8 => write!(f, "F8"),
            VirtualKeyCode::F9 => write!(f, "F9"),
            VirtualKeyCode::F10 => write!(f, "F10"),
            VirtualKeyCode::F11 => write!(f, "F11"),
            VirtualKeyCode::F12 => write!(f, "F12"),
            VirtualKeyCode::F13 => write!(f, "F13"),
            VirtualKeyCode::F14 => write!(f, "F14"),
            VirtualKeyCode::F15 => write!(f, "F15"),
            VirtualKeyCode::F16 => write!(f, "F16"),
            VirtualKeyCode::F17 => write!(f, "F17"),
            VirtualKeyCode::F18 => write!(f, "F18"),
            VirtualKeyCode::F19 => write!(f, "F19"),
            VirtualKeyCode::F20 => write!(f, "F20"),
            VirtualKeyCode::F21 => write!(f, "F21"),
            VirtualKeyCode::F22 => write!(f, "F22"),
            VirtualKeyCode::F23 => write!(f, "F23"),
            VirtualKeyCode::F24 => write!(f, "F24"),
            VirtualKeyCode::Snapshot => write!(f, "Print Screen"),
            VirtualKeyCode::Scroll => write!(f, "Scroll Lock"),
            VirtualKeyCode::Pause => write!(f, "Pause"),
            VirtualKeyCode::Insert => write!(f, "Insert"),
            VirtualKeyCode::Home => write!(f, "Home"),
            VirtualKeyCode::Delete => write!(f, "Delete"),
            VirtualKeyCode::End => write!(f, "End"),
            VirtualKeyCode::PageDown => write!(f, "Page Down"),
            VirtualKeyCode::PageUp => write!(f, "Page Up"),
            VirtualKeyCode::Left => write!(f, "Left"),
            VirtualKeyCode::Up => write!(f, "Up"),
            VirtualKeyCode::Right => write!(f, "Right"),
            VirtualKeyCode::Down => write!(f, "Down"),
            VirtualKeyCode::Back => write!(f, "Backspace"),
            VirtualKeyCode::Return => write!(f, "Enter"),
            VirtualKeyCode::Space => write!(f, "Space"),
            VirtualKeyCode::Caret => write!(f, "^"),
            VirtualKeyCode::Numlock => write!(f, "Numlock"),
            VirtualKeyCode::Numpad0 => write!(f, "Numpad 0"),
            VirtualKeyCode::Numpad1 => write!(f, "Numpad 1"),
            VirtualKeyCode::Numpad2 => write!(f, "Numpad 2"),
            VirtualKeyCode::Numpad3 => write!(f, "Numpad 3"),
            VirtualKeyCode::Numpad4 => write!(f, "Numpad 4"),
            VirtualKeyCode::Numpad5 => write!(f, "Numpad 5"),
            VirtualKeyCode::Numpad6 => write!(f, "Numpad 6"),
            VirtualKeyCode::Numpad7 => write!(f, "Numpad 7"),
            VirtualKeyCode::Numpad8 => write!(f, "Numpad 8"),
            VirtualKeyCode::Numpad9 => write!(f, "Numpad 9"),
            VirtualKeyCode::NumpadAdd => write!(f, "Numpad Add"),
            VirtualKeyCode::NumpadDivide => write!(f, "Numpad Divide"),
            VirtualKeyCode::NumpadDecimal => write!(f, "Numpad Decimal"),
            VirtualKeyCode::NumpadComma => write!(f, "Numpad Comma"),
            VirtualKeyCode::NumpadEnter => write!(f, "Numpad Enter"),
            VirtualKeyCode::NumpadEquals => write!(f, "Numpad Equals"),
            VirtualKeyCode::NumpadMultiply => write!(f, "Numpad Multiply"),
            VirtualKeyCode::NumpadSubtract => write!(f, "Numpad Subtract"),
            VirtualKeyCode::Apostrophe => write!(f, "'"),
            VirtualKeyCode::Asterisk => write!(f, "*"),
            VirtualKeyCode::Backslash => write!(f, "\\"),
            VirtualKeyCode::Colon => write!(f, ":"),
            VirtualKeyCode::Comma => write!(f, ","),
            VirtualKeyCode::Equals => write!(f, "Equals"),
            VirtualKeyCode::Grave => write!(f, "`"),
            VirtualKeyCode::Minus => write!(f, "-"),
            VirtualKeyCode::Period => write!(f, "."),
            VirtualKeyCode::Plus => write!(f, "+"),
            VirtualKeyCode::Semicolon => write!(f, ";"),
            VirtualKeyCode::Slash => write!(f, "/"),
            VirtualKeyCode::Tab => write!(f, "Tab"),

            _ => write!(f, "{:?}", self.0),
        }
    }
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Default)]
pub struct KeyCombination {
    pub key       : KeyCode,
    pub modifiers : ModifiersState,
}

impl From<(VirtualKeyCode, ModifiersState)> for KeyCombination {
    fn from((key, modifier): (VirtualKeyCode, ModifiersState)) -> Self {
        return Self { key: KeyCode(key), modifiers: modifier };
    }
}

impl Display for KeyCombination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.modifiers {
            ModifiersState::CTRL => write!(f, "Ctrl + ")?,
            ModifiersState::SHIFT => write!(f, "Shift + ")?,
            ModifiersState::ALT => write!(f, "Alt + ")?,
            _ => {}
        }

        return write!(f, "{}", self.key);
    }
}

#[derive(Clone)]
pub struct Bind<T: Clone + Copy + PartialEq + Eq + Hash> {
    pub id          : T,
    pub name        : String,
    pub description : String,
}