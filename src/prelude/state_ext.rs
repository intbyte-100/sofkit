use crate::state::StateHolder;
use gtk::glib::object::{IsA, ObjectExt};


const STATE_HOLDER_KEY: &str = "sofkit-state-holder";

pub fn statefull<T: FnOnce(&StateHolder) -> W, W: IsA<gtk::Widget>>(
    build_ui: T,
) -> impl IsA<gtk::Widget> {
    let state_holder = StateHolder::new();
    build_ui(&state_holder).attach_state_holder(state_holder)
}

pub trait StateHolderExt {
    fn attach_state_holder(self, holder: StateHolder) -> Self;
}

impl<T: IsA<gtk::Widget>> StateHolderExt for T {
    fn attach_state_holder(self, holder: StateHolder) -> Self {
        unsafe { self.set_data(STATE_HOLDER_KEY, holder) }
        self
    }
}
