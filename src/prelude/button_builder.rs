use std::fmt::Display;

use gtk::glib::WeakRef;
use gtk::glib::object::ObjectExt;
use gtk::prelude::{ButtonExt, WidgetExt};
use gtk::{Button, builders::ButtonBuilder};

use crate::state::State;

pub struct ReactiveButtonBuilder {
    on_click: Option<Box<dyn Fn()>>,
    subscribes: Vec<Box<dyn Fn(&Button)>>,
    builder: ButtonBuilder,
}

impl ReactiveButtonBuilder {
    pub fn new() -> Self {
        Self {
            on_click: None,
            subscribes: Vec::new(),
            builder: Button::builder(),
        }
    }

    pub fn on_click<T: Fn() + 'static>(mut self, on_click: T) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }

    pub fn with_raw<T>(mut self, editor: T) -> Self
    where
        T: Fn(ButtonBuilder) -> ButtonBuilder,
    {
        self.builder = editor(self.builder);
        self
    }

    pub fn label_state<T: Display + 'static, D: State<T> + 'static>(self, string: &D) -> Self {
        self.with_state(string, |button, it| {
            button.set_label(it.to_string().as_str())
        })
    }
    
    pub fn css_class_state<T: Display + 'static, D: State<T> + 'static>(self, string: &D) -> Self {
        self.with_state(string, |button, it| {
            for i in button.css_classes() {
                button.remove_css_class(i.as_str());
            }
            button.add_css_class(it.to_string().as_str())
        })
    }

    pub fn bind_state<T: Clone + 'static, S: Fn(&Button, T) + 'static + Clone, D: State<T> + 'static>(
        self,
        state: &D,
        callback: S,
    ) -> Self {
        self.with_state(state, move |button, it| {
            callback(&button, it.clone());
        })
    }
    
    pub fn with_state<T: 'static, S: Fn(Button, &T) + 'static + Clone, D: State<T> + 'static>(
        mut self,
        state: &D,
        callback: S,
    ) -> Self {
        
        let state = state.clone();
        
        self.subscribes.push(Box::new(move |button| {
            let callback = callback.clone();
            
            let button_weak = button.downgrade();
            
            state.subscribe_widget(button, move |it| {
                if let Some(button_ref) = button_weak.upgrade() {
                    callback(button_ref, it);
                }
            });
        }));   

        self
    }

    pub fn build(self) -> Button {
        let button = self.builder.build();

        if let Some(on_click) = self.on_click {
            button.connect_clicked(move |_| on_click());
        }
        
        for subscribe in self.subscribes {
            subscribe(&button);
        }

        button
    }
}

pub trait ButtonBuilderExt {
    fn reactive(self) -> ReactiveButtonBuilder;
}

impl ButtonBuilderExt for ButtonBuilder {
    fn reactive(self) -> ReactiveButtonBuilder {
        ReactiveButtonBuilder {
            on_click: None,
            builder: self,
            subscribes: Vec::new(),
        }
    }
}

pub fn button() -> ButtonBuilder {
    Button::builder()
}
