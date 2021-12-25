use hashbrown::HashMap;

use crate::message::TimedAction;

pub struct Timer(HashMap<TimedAction, f32>);

impl Timer {
    pub fn update(&mut self, delta: f32) {
        for time in self.0.values_mut() {
            *time += delta;
        }
    }

    pub fn wait(&mut self, kind: TimedAction) -> bool {
        match self.0.get(&kind) {
            Some(time) => {
                if *time > kind.duration() {
                    self.0.remove(&kind);
                    true
                } else {
                    false
                }
            }
            None => {
                self.0.insert(kind, 0.0);
                false
            }
        }
    }
}

impl TimedAction {
    fn duration(&self) -> f32 {
        match self {
            TimedAction::Selecting => 2.0,
            TimedAction::Replace => 4.0,
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self(Default::default())
    }
}
