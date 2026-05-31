use std::fmt::Display;

use gtk::glib::object::IsA;
use gtk::prelude::ButtonExt;
use gtk::{Button, builders::ButtonBuilder};

use crate::prelude::reactive_widget::ReactiveWidget;
use crate::state::ReadState;
use crate::value::ReactiveValue;

pub trait ReactiveButton<W: IsA<Button> + IsA<gtk::Widget> + 'static>:
    ReactiveWidget<W>
{
    fn label<D: ReactiveValue<String> + 'static>(self, string: D) -> Self
    where
        Self: Sized,
    {
        string.bind(self.as_widget(), |button, value| {
            button.set_label(value.as_str());
        });
        
        self
    }

    fn on_click<T: Fn() + 'static>(self, on_click: T) -> Self
    where
        Self: Sized,
    {
        self.as_widget().connect_clicked(move |_| on_click());
        self
    }
}
pub struct ReactiveButtonStruct {
    widget: Button,
}

impl ReactiveWidget<Button> for ReactiveButtonStruct {
    fn as_widget(&self) -> &Button {
        &self.widget
    }

    fn build(self) -> Button {
        self.widget
    }
}

impl ReactiveButton<Button> for ReactiveButtonStruct {}

pub trait ButtonBuilderExt {
    fn reactive(self) -> ReactiveButtonStruct;
}

impl ButtonBuilderExt for ButtonBuilder {
    fn reactive(self) -> ReactiveButtonStruct {
        ReactiveButtonStruct {
            widget: self.build(),
        }
    }
}

pub fn button() -> ReactiveButtonStruct {
    ReactiveButtonStruct {
        widget: Button::builder().build(),
    }
}
