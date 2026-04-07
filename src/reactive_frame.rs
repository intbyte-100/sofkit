use std::cell::Cell;

use gtk::glib;

struct ReactiveFrame {
    frame: Cell<u64>,
    is_updated: Cell<bool>,
}

impl ReactiveFrame {
    fn new() -> Self {
        Self {
            frame: Cell::new(0),
            is_updated: Cell::new(false),
        }
    }
}

thread_local! {
    static REACTIVE_FRAME: ReactiveFrame = ReactiveFrame::new();
}

pub(crate) fn current_reactive_frame() -> u64 {
    REACTIVE_FRAME.with(|it| {
        if !it.is_updated.get() {
            it.frame.set(it.frame.get() + 1);
            it.is_updated.set(true);
            
            glib::idle_add_local_once(move || {
                REACTIVE_FRAME.with(|it| {
                    it.is_updated.set(false);
                });
            });
        }

        it.frame.get()
    })
}
