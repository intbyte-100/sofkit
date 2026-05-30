use gtk::{
    Button, Widget,
    glib::object::{IsA, ObjectExt},
    prelude::WidgetExt,
};

use crate::state::{ReadState, StateAccessor};

pub trait ReactiveBuilder<T: IsA<Widget>> {
    fn as_widget(&self) -> &T;

    fn build(self) -> T;

    fn bind<V: 'static, S: ReadState<V>>(
        self,
        state: &S,
        callback: impl Fn(&T, &StateAccessor<V>) + 'static,
    ) -> Self
    where
        Self: Sized,
    {
        let widget = self.as_widget().downgrade();
        state.subscribe_widget(self.as_widget(), move |v| {
            if let Some(widget) = widget.upgrade() {
                callback(&widget, v);
            }
        });
        self
    }

    fn visible<S: ReadState<bool>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_visible(v.get()))
    }

    fn opacity<S: ReadState<f64>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_opacity(v.get()))
    }

    fn sensitive<S: ReadState<bool>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_sensitive(v.get()))
    }

    fn tooltip_text<S: ReadState<String>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| {
            v.with(|v| widget.set_tooltip_text(Some(v.as_str())))
        })
    }

    fn css_classes<S: ReadState<Vec<String>>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| {
            v.with(|it| {
                let classes = it.iter().map(|c| c.as_str()).collect::<Vec<_>>();
                widget.set_css_classes(&classes);
            })
        })
    }

    fn hexpand<S: ReadState<bool>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_hexpand(v.get()))
    }

    fn vexpand<S: ReadState<bool>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_vexpand(v.get()))
    }

    fn halign<S: ReadState<gtk::Align>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_halign(v.get()))
    }

    fn valign<S: ReadState<gtk::Align>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_valign(v.get()))
    }

    fn margin_start<S: ReadState<i32>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_margin_start(v.get()))
    }

    fn margin_end<S: ReadState<i32>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_margin_end(v.get()))
    }

    fn margin_top<S: ReadState<i32>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_margin_top(v.get()))
    }

    fn margin_bottom<S: ReadState<i32>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_margin_bottom(v.get()))
    }

    fn margin_all<S: ReadState<i32>>(self, state: &S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| {
            v.with(|value| {
                widget.set_margin_start(*value);
                widget.set_margin_end(*value);
                widget.set_margin_top(*value);
                widget.set_margin_bottom(*value);
            })
        })
    }
}
