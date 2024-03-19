use std::collections::HashSet;

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputContext {
    pressed: HashSet<PhysicalKey>,
    just_pressed: HashSet<PhysicalKey>,
    released: HashSet<PhysicalKey>,
}

impl InputContext {
    pub(super) fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            released: HashSet::new(),
        }
    }

    pub(in crate::engine) fn insert(&mut self, event: KeyEvent) {
        let phys = event.physical_key;
        let state = event.state;
        let repeat = event.repeat;

        match state {
            ElementState::Pressed => {
                if !repeat {
                    self.just_pressed.insert(phys);
                }

                self.pressed.insert(phys);
            }
            ElementState::Released => {
                self.pressed.remove(&phys);
                self.released.insert(phys);
            }
        }
    }

    pub(in crate::engine) fn update(&mut self) {
        self.just_pressed.clear();
        self.released.clear();
    }

    fn from_code(code: KeyCode) -> PhysicalKey {
        PhysicalKey::Code(code)
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        let phys = Self::from_code(key);
        self.pressed.contains(&phys)
    }

    pub fn is_keys_pressed(&self, keys: &[KeyCode]) -> bool {
        for key in keys {
            let phys = Self::from_code(*key);
            if !self.pressed.contains(&phys) {
                return false;
            }
        }
        return true;
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        let phys = Self::from_code(key);
        self.just_pressed.contains(&phys)
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        let phys = Self::from_code(key);
        self.released.contains(&phys)
    }
}