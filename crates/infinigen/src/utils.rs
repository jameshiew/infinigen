use std::collections::VecDeque;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Queue<T> {
    inner: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn push_back(&mut self, item: T) {
        self.inner.push_back(item);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
