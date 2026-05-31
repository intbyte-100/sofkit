use std::fmt::Display;

use gtk::Label;
use gtk::prelude::*;

use crate::prelude::reactive_widget::ReactiveWidget;
use crate::runtime::Runtime;
use crate::value::ReactiveValue;

pub trait ReactiveLabel: ReactiveWidget<Label> {
    fn text<D: ReactiveValue<T> + 'static, T: Display + 'static>(self, state: D) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |label, value| {
            label.set_label(value.to_string().as_str());
        })
    }
}

pub struct ReactiveLabelStruct {
    widget: Label,
}

impl ReactiveWidget<Label> for ReactiveLabelStruct {
    fn as_widget(&self) -> &Label {
        &self.widget
    }

    fn build(self) -> Label {
        self.widget
    }
}

impl ReactiveLabel for ReactiveLabelStruct {}

pub fn label<D: ReactiveValue<T> + 'static, T: Display + 'static>(text: D) -> ReactiveLabelStruct {
    let widget = Label::new(None);
    Runtime::get().bind_widget(&widget);
    ReactiveLabelStruct { widget }.text(text)
}
