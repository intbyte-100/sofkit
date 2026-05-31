use gtk::Label;
use gtk::prelude::*;

use crate::prelude::reactive_widget::ReactiveWidget;
use crate::runtime::Runtime;
use crate::value::ReactiveValue;

pub trait ReactiveLabel: ReactiveWidget<Label> {
    fn text<D: ReactiveValue<String> + 'static>(self, state: D) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |label, value| {
            label.set_label(value.as_str());
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

pub fn label<D: ReactiveValue<String> + 'static>(text: D) -> ReactiveLabelStruct {
    let widget = Label::new(None);
    Runtime::get().bind_widget(&widget);
    ReactiveLabelStruct { widget }.text(text)
}
