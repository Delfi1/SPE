use std::collections::HashSet;

use winit::event::ElementState;

pub struct InputContext {
    pressed_keys: HashSet<u32>,
    just_pressed_keys: HashSet<u32>,
    released_keys: HashSet<u32>
}

impl InputContext {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            released_keys: HashSet::new()
        }
    }

    pub fn insert(&mut self, code: u32, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if !self.pressed_keys.contains(&code) {
                    self.just_pressed_keys.insert(code);
                    println!("Key {code} was just pressed");
                }

                self.pressed_keys.insert(code);
            },
            ElementState::Released => {
                self.pressed_keys.remove(&code);
                self.released_keys.insert(code);
            }
        }
    }

    pub fn is_key_pressed(&self, code: u32) -> bool {
        self.pressed_keys.contains(&code)
    }

    pub fn is_keys_pressed(&self, codes: &[u32]) -> bool {
        for code in codes {
            if !self.pressed_keys.contains(code) {
                return false;
            }
        }
        return true;
    }

    pub fn is_key_released(&self, code: u32) -> bool {
        self.released_keys.contains(&code)
    }

    pub fn is_key_just_pressed(&self, code: u32) -> bool {
        self.just_pressed_keys.contains(&code)
    }

    pub(in crate::engine) fn update(&mut self) {
        self.released_keys.clear();
        self.just_pressed_keys.clear();
    }
}
