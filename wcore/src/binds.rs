use std::{fmt::Display, collections::HashMap, hash::Hash};

use ahash::RandomState;
use winit::event::{VirtualKeyCode, ModifiersState};

pub type Keybinds<T> = HashMap<KeyCombination, Keybind<T>, RandomState>;

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct KeyCode(VirtualKeyCode);

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

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct KeyCombination {
    pub key: KeyCode,
    pub modifier: ModifiersState,
}

impl From<(VirtualKeyCode, ModifiersState)> for KeyCombination {
    fn from((key, modifier): (VirtualKeyCode, ModifiersState)) -> Self {
        return Self { key: KeyCode(key), modifier };
    }
}

impl Display for KeyCombination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.modifier {
            ModifiersState::CTRL => write!(f, "Ctrl + ")?,
            ModifiersState::SHIFT => write!(f, "Shift + ")?,
            ModifiersState::ALT => write!(f, "Alt + ")?,
            _ => {}
        }

        return write!(f, "{}", self.key);
    }
}

pub struct Keybind<T: Clone + Copy + PartialEq + Eq + Hash> {
    pub id          : T,
    pub name        : String,
    pub description : String,
}