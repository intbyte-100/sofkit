use gtk::Entry;
use gtk::prelude::*;

use crate::prelude::reactive_widget::ReactiveWidget;
use crate::runtime::Runtime;
use crate::state::{ReadState, StateAccessor, StateHandle};

pub trait ReactiveEntry: ReactiveWidget<Entry> {
    fn text<T: ReadState<String> + 'static>(self, state: &T) -> Self
    where
        Self: Sized,
    {
        let state = state.clone();
        let widget = self.as_widget().downgrade();
        state.subscribe_widget(self.as_widget(), move |it: &StateAccessor<String>| {
            if let Some(entry) = widget.upgrade() {
                let new_text = it.get();
                if entry.text().as_str() != new_text.as_str() {
                    entry.set_text(new_text.as_str());
                }
            }
        });
        self
    }

    fn text_two_way(self, state: StateHandle<String>) -> Self
    where
        Self: Sized,
    {
        let s_for_sub = state.clone();
        let self_after_bind = self.text(&s_for_sub);

        let state2 = state.clone();
        self_after_bind.as_widget().connect_changed(move |e| {
            let text = e.text();
            let update = state2
                .with(|it| it.as_str() != text.as_str())
                .unwrap_or(false);
            if update {
                state2.set(text.to_string());
            }
        });

        self_after_bind
    }

    fn on_changed<T: Fn(String) + 'static>(self, cb: T) -> Self
    where
        Self: Sized,
    {
        self.as_widget()
            .connect_changed(move |e| cb(e.text().to_string()));
        self
    }
}

pub struct ReactiveEntryStruct {
    widget: Entry,
}

impl ReactiveWidget<Entry> for ReactiveEntryStruct {
    fn as_widget(&self) -> &Entry {
        &self.widget
    }

    fn build(self) -> Entry {
        self.widget
    }
}

impl ReactiveEntry for ReactiveEntryStruct {}

pub fn entry() -> ReactiveEntryStruct {
    let widget = Entry::new();
    Runtime::get().bind_widget(&widget);
    ReactiveEntryStruct { widget }
}
