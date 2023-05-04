mod utils;

use crdt;
use crdt::clocks::S4Vector;
use crdt::data_structure::{Operation, SynchronizedText};
use wasm_bindgen::prelude::*;

extern crate serde_json;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, World!");
}

#[wasm_bindgen]
pub struct TextBoxSynchronizer {
    text: SynchronizedText,
    cursor_pos: crdt::clocks::S4Vector,
}
#[wasm_bindgen]
impl TextBoxSynchronizer {

    pub fn new(id: usize) -> TextBoxSynchronizer {
        TextBoxSynchronizer {
            text: SynchronizedText::new(id),
            cursor_pos: crdt::clocks::S4Vector::root(),
        }
    }

    pub fn get_text(&self)-> String {
        self.text.get_text()

    }

    pub fn insert_at_cursor(&mut self, character: char) -> String {
        let op = self.text.local_insert(self.cursor_pos, character);
        self.cursor_pos = self.text.get_clock().to_s4vector();
        serde_json::to_string(&op).unwrap()
    }

    pub fn apply_remote_operation(&mut self, operation: &str) {
        let op: Operation = serde_json::de::from_str(operation).unwrap();
        self.text.apply_operation(op).unwrap();
    }

    pub fn get_absolute_cursor_pos(&self) -> usize {
        let mut result = 0;
        for (pos, c) in self.text.iter() {
            if c.is_some() {
                result += 1;
            }
            if pos == self.cursor_pos {
                break
            }
        }
        result
    }

    pub fn set_absolute_cursor_pos(&mut self, pos: usize) {
        let mut current_absolute_pos = 0;
        self.cursor_pos = S4Vector::root();
        for (s4_pos, c) in self.text.iter() {
            if c.is_some() {
                current_absolute_pos += 1;
            }
            if pos == current_absolute_pos {
                self.cursor_pos = s4_pos;
                break
            }
        }
    }
}
