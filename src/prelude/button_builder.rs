use std::fmt::Display;

use gtk::glib::object::IsA;
use gtk::prelude::ButtonExt;
use gtk::{Button, builders::ButtonBuilder};

use crate::prelude::reactive_builder::ReactiveBuilder;
use crate::state::ReadState;

pub trait ReactiveButtonBuilder<W: IsA<Button> + IsA<gtk::Widget> + 'static>:
    ReactiveBuilder<W>
{
    fn label<T: Display + 'static, D: ReadState<T> + 'static>(self, string: &D) -> Self
    where
        Self: Sized,
    {
        self.bind(string, |button, it| {
            button.set_label(it.with(|it| it.to_string()).as_str())
        })
    }

    fn on_click<T: Fn() + 'static>(self, on_click: T) -> Self
    where
        Self: Sized,
    {
        self.as_widget().connect_clicked(move |_| on_click());
        self
    }
}
pub struct ReactiveButtonBuilderStruct {
    widget: Button,
}

impl ReactiveBuilder<Button> for ReactiveButtonBuilderStruct {
    fn as_widget(&self) -> &Button {
        &self.widget
    }

    fn build(self) -> Button {
        self.widget
    }
}

impl ReactiveButtonBuilder<Button> for ReactiveButtonBuilderStruct {}

pub trait ButtonBuilderExt {
    fn reactive(self) -> ReactiveButtonBuilderStruct;
}

impl ButtonBuilderExt for ButtonBuilder {
    fn reactive(self) -> ReactiveButtonBuilderStruct {
        ReactiveButtonBuilderStruct {
            widget: self.build(),
        }
    }
}

pub fn button() -> ButtonBuilder {
    Button::builder()
}
