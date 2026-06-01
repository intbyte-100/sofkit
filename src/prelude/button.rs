use gtk::Button;
use gtk::glib::object::IsA;
use gtk::prelude::ButtonExt;

use crate::prelude::reactive_widget::ReactiveWidget;
use crate::runtime::Runtime;
use crate::value::ReactiveValue;

pub trait ReactiveButton<W: IsA<Button> + IsA<gtk::Widget> + 'static>: ReactiveWidget<W> {
    fn label<D: ReactiveValue<String> + 'static>(self, string: D) -> Self
    where
        Self: Sized,
    {
        string.bind(self.as_widget(), |button, value| {
            value.with(|value| button.set_label(value));
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

impl ReactiveButton<Button> for ReactiveButtonStruct {}

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

pub fn button() -> ReactiveButtonStruct {
    let widget = Button::new();

    Runtime::get().bind_widget(&widget);

    ReactiveButtonStruct { widget }
}
