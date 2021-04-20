#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::rot_layerbase::Layer;
use std::collections::HashMap;

#[derive(Default)]
pub struct LayerStack {
    stack: Vec<Box<dyn Layer>>,
    dictionary: HashMap<String, usize>,
}

impl LayerStack {
    pub fn new() -> Self {
        LayerStack {
            stack: Vec::new(),
            dictionary: HashMap::new(),
        }
    }

    pub fn push_indexed(&mut self, layer: Box<dyn Layer>, index: usize) {
        let len = self.stack.len() + 1;

        if index > self.stack.len() {
            self.stack.push(layer);

            self.dictionary.insert(
                self.stack[len].as_ref().get_name().clone(),
                self.stack.len(),
            );
        } else {
            self.stack.insert(index, layer);
            self.dictionary
                .insert(self.stack[index].as_ref().get_name().clone(), index);
        }
    }

    pub fn pop_indexed(&mut self, index: usize) -> Option<Box<dyn Layer>> {
        if index > self.stack.len() {
            warn!("Trying to pop Layerstack beyond limits. Notting happened");
            None
        } else {
            Some(self.stack.remove(index))
        }
    }

    pub fn get_named(&self, name: &str) -> &dyn Layer {
        self.stack[self.dictionary[name]].as_ref()
    }

    pub fn stack(&mut self) -> &mut Vec<Box<dyn Layer>> {
        &mut self.stack
    }
}
