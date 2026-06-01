use gtk::CheckButton;
use gtk::prelude::*;

use crate::prelude::reactive_widget::ReactiveWidget;
use crate::runtime::Runtime;
use crate::state::State;
use crate::state::{ReadState, StateAccessor, WriteState};

pub trait ReactiveCheckButton: ReactiveWidget<CheckButton> {
    fn active<T: ReadState<bool> + 'static>(self, state: &T) -> Self
    where
        Self: Sized,
    {
        let state = state.clone();
        let widget = self.as_widget().downgrade();
        state.subscribe_widget(self.as_widget(), move |it: &StateAccessor<bool>| {
            if let Some(cb) = widget.upgrade()
                && cb.is_active() != it.get()
            {
                cb.set_active(it.get());
            }
        });
        self
    }

    fn toggle<S: State<bool> + 'static>(self, state: S) -> Self
    where
        Self: Sized,
    {
        let self_after_bind = self.active(&state);
        let widget = self_after_bind.as_widget();

        widget.connect_toggled(move |c| {
            let active = c.is_active();
            state.edit(move |it| *it = active);
        });
        self_after_bind
    }

    fn on_toggled<T: Fn(bool) + 'static>(self, cb: T) -> Self
    where
        Self: Sized,
    {
        self.as_widget().connect_toggled(move |c| cb(c.is_active()));
        self
    }
}

pub struct ReactiveCheckButtonStruct {
    widget: CheckButton,
}

impl ReactiveWidget<CheckButton> for ReactiveCheckButtonStruct {
    fn as_widget(&self) -> &CheckButton {
        &self.widget
    }

    fn build(self) -> CheckButton {
        self.widget
    }
}

impl ReactiveCheckButton for ReactiveCheckButtonStruct {}

pub fn check_button() -> ReactiveCheckButtonStruct {
    let widget = CheckButton::new();
    Runtime::get().bind_widget(&widget);
    ReactiveCheckButtonStruct { widget }
}
