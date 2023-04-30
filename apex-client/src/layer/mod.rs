use wcore::{binds::KeyCombination, graphics::layer::Layer};

use crate::state::AppState;

use self::taiko::TaikoLayer;

pub mod taiko;

pub struct Layers {
    pub taiko : TaikoLayer,
}

impl Layers {
    pub fn action(&mut self, keys: KeyCombination, state: &mut AppState) -> bool {
        // Consumes input
        if self.taiko.action(keys, &mut state.taiko) { return true };

        // Passes input
        return false;
    }
}