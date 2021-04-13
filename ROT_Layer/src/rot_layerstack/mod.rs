#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use log::{debug, error, info, trace, warn};

use crate::rot_layerbase::ROT_Layer;
use std::collections::HashMap;
use std::sync::Arc;

pub struct LayerStack {
    stack: Vec<Box<dyn ROT_Layer>>,
    dictionary: HashMap<String, usize>,
}

impl LayerStack {
    pub fn new() -> Self {
        LayerStack {
            stack: Vec::new(),
            dictionary: HashMap::new(),
        }
    }

    pub fn push_indexed(&mut self, layer: Box<dyn ROT_Layer>, index: usize) {
        let len = self.stack.len() + 1;

        if index > self.stack.len() {
            self.stack.push(layer);
            self.stack[len].as_mut().assign_index(len);
            self.dictionary.insert(
                self.stack[len].as_ref().get_name().clone(),
                self.stack.len(),
            );
        } else {
            self.stack.insert(index, layer);
            self.stack[index].as_mut().assign_index(index);
            self.dictionary
                .insert(self.stack[index].as_ref().get_name().clone(), index);
        }
    }

    pub fn pop_indexed(&mut self, index: usize) -> Option<Box<dyn ROT_Layer>> {
        if index > self.stack.len() {
            warn!("Trying to pop Layerstack beyond limits. Notting happened");
            None
        } else {
            self.stack[index].as_mut().disable_index();
            Some(self.stack.remove(index))
        }
    }

    pub fn get_named(&self, name: &String) -> &dyn ROT_Layer {
        self.stack[self.dictionary[name]].as_ref()
    }

    pub fn stack(&mut self) -> &mut Vec<Box<dyn ROT_Layer>> {
        &mut self.stack
    }
}
