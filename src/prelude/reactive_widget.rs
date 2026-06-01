use gtk::{
    Widget,
    glib::object::{IsA, ObjectExt},
    prelude::WidgetExt,
};

use crate::value::ReactiveValue;

pub trait ReactiveWidget<T: IsA<Widget>> {
    fn as_widget(&self) -> &T;

    fn build(self) -> T;

    fn bind<V: 'static, S: ReactiveValue<V>>(
        self,
        state: S,
        callback: impl Fn(&T, &V) + 'static,
    ) -> Self
    where
        Self: Sized,
    {
        state.bind(self.as_widget(), move |widget, value| {
            value.with(|v| callback(widget, v))
        });
        self
    }

    fn visible<S: ReactiveValue<bool>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_visible(*v))
    }

    fn opacity<S: ReactiveValue<f64>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_opacity(*v))
    }

    fn sensitive<S: ReactiveValue<bool>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_sensitive(*v))
    }

    fn tooltip_text<S: ReactiveValue<String>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_tooltip_text(Some(v.as_str())))
    }

    fn css_classes<S: ReactiveValue<Vec<String>>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| {
            let classes = v.iter().map(|c| c.as_str()).collect::<Vec<_>>();
            widget.set_css_classes(&classes);
        })
    }

    fn css_class<S: ReactiveValue<String>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| {
            widget.set_css_classes(&[v.as_str()]);
        })
    }
    
    fn hexpand<S: ReactiveValue<bool>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_hexpand(*v))
    }

    fn vexpand<S: ReactiveValue<bool>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_vexpand(*v))
    }

    fn halign<S: ReactiveValue<gtk::Align>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_halign(*v))
    }

    fn valign<S: ReactiveValue<gtk::Align>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_valign(*v))
    }

    fn margin_start<S: ReactiveValue<i32>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_margin_start(*v))
    }

    fn margin_end<S: ReactiveValue<i32>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_margin_end(*v))
    }

    fn margin_top<S: ReactiveValue<i32>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_margin_top(*v))
    }

    fn margin_bottom<S: ReactiveValue<i32>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| widget.set_margin_bottom(*v))
    }

    fn margin_all<S: ReactiveValue<i32>>(self, state: S) -> Self
    where
        Self: Sized,
    {
        self.bind(state, |widget, v| {
            let value = *v;
            widget.set_margin_start(value);
            widget.set_margin_end(value);
            widget.set_margin_top(value);
            widget.set_margin_bottom(value);
        })
    }
}
