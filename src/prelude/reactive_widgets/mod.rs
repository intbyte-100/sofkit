use std::fmt::Display;

use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::{
    CheckButton, Entry, Label,
    builders::{CheckButtonBuilder, EntryBuilder, LabelBuilder},
};

use crate::state::{State, StateHandle};

pub struct ReactiveLabelBuilder {
    subscribes: Vec<Box<dyn Fn(&Label)>>,
    builder: LabelBuilder,
}

impl ReactiveLabelBuilder {
    pub fn new() -> Self {
        Self {
            subscribes: Vec::new(),
            builder: Label::builder(),
        }
    }

    pub fn with_raw<T>(mut self, editor: T) -> Self
    where
        T: Fn(LabelBuilder) -> LabelBuilder,
    {
        self.builder = editor(self.builder);
        self
    }

    pub fn text_state<T: Display + 'static, D: State<T> + 'static>(self, state: &D) -> Self {
        self.bind_state(state, |label, it| label.set_label(it.to_string().as_str()))
    }

    pub fn bind_state<T: 'static, S: Fn(Label, &T) + 'static + Clone, D: State<T> + 'static>(
        mut self,
        state: &D,
        callback: S,
    ) -> Self {
        let state = state.clone();
        self.subscribes.push(Box::new(move |label| {
            let callback = callback.clone();
            let label_weak = label.downgrade();
            state.subscribe_widget(label, move |it| {
                if let Some(label) = label_weak.upgrade() {
                    callback(label, it);
                }
            });
        }));
        self
    }

    pub fn build(self) -> Label {
        let label = self.builder.build();
        for subscribe in self.subscribes {
            subscribe(&label);
        }
        label
    }
}

pub struct ReactiveEntryBuilder {
    subscribes: Vec<Box<dyn Fn(&Entry)>>,
    builder: EntryBuilder,
    on_change_callbacks: Vec<Box<dyn Fn(String)>>,
    two_way_state: Option<StateHandle<String>>,
}

impl ReactiveEntryBuilder {
    pub fn new() -> Self {
        Self {
            subscribes: Vec::new(),
            builder: Entry::builder(),
            on_change_callbacks: Vec::new(),
            two_way_state: None,
        }
    }

    pub fn with_raw<T>(mut self, editor: T) -> Self
    where
        T: Fn(EntryBuilder) -> EntryBuilder,
    {
        self.builder = editor(self.builder);
        self
    }

    pub fn text_state<T: State<String> + 'static>(mut self, state: &T) -> Self {
        self.bind_state(state, |entry, it| {
            if it.as_str() != entry.text().as_str() {
                entry.set_text(it.as_str())
            }
        })
    }

    pub fn bind_state<T: 'static, S: Fn(Entry, &T) + 'static + Clone, D: State<T> + 'static>(
        mut self,
        state: &D,
        callback: S,
    ) -> Self {
        let state = state.clone();
        self.subscribes.push(Box::new(move |entry| {
            let callback = callback.clone();
            let entry_weak = entry.downgrade();

            state.subscribe_widget(entry, move |it| {
                if let Some(entry) = entry_weak.upgrade() {
                    callback(entry, it);
                }
            });
        }));

        self
    }

    pub fn bind_state_two_way(mut self, state: StateHandle<String>) -> Self {
        let state_for_sub = state.clone();

        self.two_way_state = Some(state);
        self.text_state(&state_for_sub)
    }

    pub fn on_changed<T: Fn(String) + 'static>(mut self, cb: T) -> Self {
        self.on_change_callbacks.push(Box::new(cb));
        self
    }

    pub fn build(mut self) -> Entry {
        let entry = self.builder.build();

        let callbacks = std::mem::take(&mut self.on_change_callbacks);

        if let Some(state) = self.two_way_state.take() {
            entry.connect_changed(move |e| {
                let text = e.text();
                if let Some(borrowed) = state.get() {
                    let update = if let Ok(borrowed) = borrowed.try_borrow() {
                        borrowed.as_str() != text.as_str()
                    } else {
                        false
                    };

                    if update {
                        state.set(text.to_string());
                    }
                }
            });
        } else if !callbacks.is_empty() {
            entry.connect_changed(move |e| {
                let text = e.text().to_string();
                for cb in &callbacks {
                    cb(text.clone());
                }
            });
        }

        for subscribe in self.subscribes {
            subscribe(&entry);
        }

        entry
    }
}

pub struct ReactiveCheckButtonBuilder {
    subscribes: Vec<Box<dyn Fn(&CheckButton)>>,
    builder: CheckButtonBuilder,
    on_toggled_callbacks: Vec<Box<dyn Fn(bool)>>,
    two_way_state: Option<StateHandle<bool>>,
}

impl ReactiveCheckButtonBuilder {
    pub fn new() -> Self {
        Self {
            subscribes: Vec::new(),
            builder: CheckButton::builder(),
            on_toggled_callbacks: Vec::new(),
            two_way_state: None,
        }
    }

    pub fn with_raw<T>(mut self, editor: T) -> Self
    where
        T: Fn(CheckButtonBuilder) -> CheckButtonBuilder,
    {
        self.builder = editor(self.builder);
        self
    }

    pub fn active_state<T: State<bool> + 'static>(mut self, state: &T) -> Self {
        self.bind_state(state, |cb, it| cb.set_active(*it))
    }

    pub fn bind_state<
        T: 'static,
        S: Fn(CheckButton, &T) + 'static + Clone,
        D: State<T> + 'static,
    >(
        mut self,
        state: &D,
        callback: S,
    ) -> Self {
        let state = state.clone();
        self.subscribes.push(Box::new(move |check_button| {
            let callback = callback.clone();
            let cb_weak = check_button.downgrade();
            state.subscribe_widget(check_button, move |it| {
                if let Some(cb) = cb_weak.upgrade() {
                    callback(cb, it);
                }
            });
        }));
        self
    }

    pub fn on_toggled_state(mut self, state: StateHandle<bool>) -> Self {
        let state_for_sub = state.clone();

        self.two_way_state = Some(state);
        self.bind_state(&state_for_sub, |cb, it| {
            cb.set_active(*it);
        })
    }

    pub fn on_toggled<T: Fn(bool) + 'static>(mut self, cb: T) -> Self {
        self.on_toggled_callbacks.push(Box::new(cb));
        self
    }

    pub fn build(mut self) -> CheckButton {
        let check = self.builder.build();

        let callbacks = std::mem::take(&mut self.on_toggled_callbacks);

        if let Some(state) = self.two_way_state.take() {
            check.connect_toggled(move |c| {
                let active = c.is_active();
                state.edit(move |it| *it = active);
                for cb in &callbacks {
                    cb(active);
                }
            });
        } else if !callbacks.is_empty() {
            check.connect_toggled(move |c| {
                let active = c.is_active();
                for cb in &callbacks {
                    cb(active);
                }
            });
        }

        for subscribe in self.subscribes {
            subscribe(&check);
        }

        check
    }
}

pub fn label() -> LabelBuilder {
    Label::builder()
}

pub fn entry() -> EntryBuilder {
    Entry::builder()
}

pub fn check_button() -> CheckButtonBuilder {
    CheckButton::builder()
}

pub trait LabelBuilderExt {
    fn reactive(self) -> ReactiveLabelBuilder;
}

impl LabelBuilderExt for LabelBuilder {
    fn reactive(self) -> ReactiveLabelBuilder {
        ReactiveLabelBuilder {
            subscribes: Vec::new(),
            builder: self,
        }
    }
}

pub trait EntryBuilderExt {
    fn reactive(self) -> ReactiveEntryBuilder;
}

impl EntryBuilderExt for EntryBuilder {
    fn reactive(self) -> ReactiveEntryBuilder {
        ReactiveEntryBuilder {
            subscribes: Vec::new(),
            builder: self,
            on_change_callbacks: Vec::new(),
            two_way_state: None,
        }
    }
}

pub trait CheckButtonBuilderExt {
    fn reactive(self) -> ReactiveCheckButtonBuilder;
}

impl CheckButtonBuilderExt for CheckButtonBuilder {
    fn reactive(self) -> ReactiveCheckButtonBuilder {
        ReactiveCheckButtonBuilder {
            subscribes: Vec::new(),
            builder: self,
            on_toggled_callbacks: Vec::new(),
            two_way_state: None,
        }
    }
}
