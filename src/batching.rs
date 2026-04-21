use std::cell::Cell;

use crate::scheduler::Scheduler;

pub struct BatchGate {
    last_notify_pass: Cell<u64>,
}

impl BatchGate {
    pub fn new() -> Self {
        Self {
            last_notify_pass: Cell::new(0),
        }
    }

    #[inline]
    pub fn should_run(&self) -> bool {
        let current = Scheduler::get().notify_pass();

        let last = self.last_notify_pass.get();

        if current != last {
            self.last_notify_pass.set(current);
            true
        } else {
            false
        }
    }
}
