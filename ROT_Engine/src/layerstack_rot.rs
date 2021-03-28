use log::{debug, error, info, trace, warn};

use super::layer_rot::Layer;
use std::collections::HashMap;

pub struct LayerStack {
    stack: Vec<Box<Layer>>,
    dictionary: HashMap<String, usize>,
}

impl LayerStack {
    pub fn new() -> Self {
        LayerStack {
            stack: Vec::new(),
            dictionary: HashMap::new(),
        }
    }

    pub fn push_indexed(&mut self, layer: &mut Layer, index: usize) {
        if index > self.stack.len() {
            layer.assign_index(self.stack.len());
            self.dictionary
                .insert(layer.get_name(), layer.get_index().unwrap());
            self.stack.push(Box::new(layer.clone()));
        } else {
            layer.assign_index(index);
            self.dictionary
                .insert(layer.get_name(), layer.get_index().unwrap());
            self.stack.insert(index, Box::new(layer.clone()))
        }
    }

    pub fn pop_indexed(&mut self, index: usize) {
        if index > self.stack.len() {
            return;
        }
        self.stack.remove(index).disable_index()
    }

    pub fn get_named(&self, name: &String) -> &Layer {
        self.stack[self.dictionary[name]].as_ref()
    }

    pub fn stack(&self) -> &Vec<Box<Layer>> {
        &self.stack
    }
}
