use std::collections::HashSet;
use winit::event::ElementState;

pub struct InputContext {
    pressed_keys: HashSet<u32>,
    just_pressed_keys: HashSet<u32>,
    released_keys: HashSet<u32>
}

impl InputContext {
    pub(crate) fn new() -> Self {
        let pressed_keys = HashSet::new();
        let just_pressed_keys = HashSet::new();
        let released_keys = HashSet::new();

        Self { pressed_keys, just_pressed_keys, released_keys }
    }

    pub(crate) fn update(&mut self) {
        self.just_pressed_keys.clear();
        self.released_keys.clear();
    }

    pub fn is_key_pressed(&self, key: u32) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_keys_pressed(&self, keys: HashSet<u32>) -> bool {
        for key in keys {
            if !self.pressed_keys.contains(&key) {
                return false;
            }
        }
        return true;
    }

    pub fn is_key_just_pressed(&self, key: u32) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    pub fn is_key_released(&self, key: u32) -> bool {
        self.released_keys.contains(&key)
    }

    pub(crate) fn insert(&mut self, scancode: Option<u32>, state: ElementState) {
        if scancode.is_none() {
            return;
        }

        let key = scancode.unwrap();

        if state.is_pressed() {
            if !self.pressed_keys.contains(&key) {
                self.just_pressed_keys.insert(key.clone());
                //println!("Key {:?} has been just pressed;", key);
            }

            self.pressed_keys.insert(key.clone());
            //println!("Key {:?}: \"{:?}\"", key, text)
        } else {
            self.pressed_keys.remove(&key);
            self.released_keys.insert(key.clone());
            //println!("Key {:?} has been released;", key)
        }
    }
}