use crate::state::{StateHolder, Subscription, SubscriptionHolder};
use gtk::glib::object::{IsA, ObjectExt};

const STATE_HOLDER_KEY: &str = "sofkit-state-holder";
const SUBSCRIPTION_KEY: &str = "sofkit-state-holder";

pub fn statefull<T: FnOnce(&StateHolder) -> W, W: IsA<gtk::Widget>>(
    build_ui: T,
) -> impl IsA<gtk::Widget> {
    let state_holder = StateHolder::new();

    let ui = build_ui(&state_holder);
    ui.attach_state_holder(state_holder);
    ui
}

pub trait StateHolderExt {
    fn attach_state_holder(&self, holder: StateHolder);
    fn attach_subscription(&self, subscription: Subscription);
}

impl<T: IsA<gtk::Widget>> StateHolderExt for T {
    fn attach_state_holder(&self, holder: StateHolder) {
        unsafe { self.set_data(STATE_HOLDER_KEY, holder) }
    }

    fn attach_subscription(&self, subscription: Subscription) {
        unsafe {
            if let Some(holder) = self.data::<SubscriptionHolder>(SUBSCRIPTION_KEY) {
                holder.as_ref().attach_subscription(subscription);
            } else {
                let holder = SubscriptionHolder::new();
                holder.attach_subscription(subscription);
                self.set_data(SUBSCRIPTION_KEY, holder)
            }
        }
    }
}
