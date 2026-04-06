use std::cell::Cell;

use gtk::glib;



thread_local! {
    static CURRENT_FRAME: Cell<u64> = Cell::new(0);
}


pub(crate) fn current_frame() -> u64 {
    CURRENT_FRAME.with(|it| it.get())
}

pub fn init() {
    glib::idle_add_local(|| {
        CURRENT_FRAME.with(|it| {
            it.set(it.get() + 1);
            glib::ControlFlow::Continue
        })
    });
}