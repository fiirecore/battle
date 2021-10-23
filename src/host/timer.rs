use std::time::Instant;

use hashbrown::HashMap;

use crate::message::TimedAction;

pub struct Timer(HashMap<TimedAction, Instant>);

impl Timer {

    pub fn wait(&mut self, kind: TimedAction) -> bool {
        match self.0.get_mut(&kind) {
            Some(time) => if time.elapsed().as_secs_f32() > kind.duration() {
                self.0.remove(&kind);
                true
            } else {
                false
            },
            None => {
                self.0.insert(kind, Instant::now());
                false
            },
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