use gtk::glib::object::{IsA, ObjectExt};

use crate::state::{ReadState, StateAccessor};

pub trait ReactiveValue<T> {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<T>) + 'static;
}

impl ReactiveValue<String> for String {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<String>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl ReactiveValue<String> for &str {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<String>),
    {
        let accessor = StateAccessor::new(self.to_string());
        f(widget, &accessor)
    }
}

impl ReactiveValue<String> for &String {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<String>),
    {
        let accessor = StateAccessor::new(self.clone());
        f(widget, &accessor)
    }
}

impl ReactiveValue<i32> for i32 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<i32>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl ReactiveValue<bool> for bool {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<bool>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl ReactiveValue<f64> for f64 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<f64>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl ReactiveValue<f32> for f32 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<f32>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl ReactiveValue<usize> for usize {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<usize>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl ReactiveValue<isize> for isize {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<isize>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl ReactiveValue<u32> for u32 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<u32>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl ReactiveValue<i64> for i64 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<i64>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl ReactiveValue<u64> for u64 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<u64>),
    {
        let accessor = StateAccessor::new(self);
        f(widget, &accessor)
    }
}

impl<T, S> ReactiveValue<T> for S
where
    S: ReadState<T>,
    T: Clone + 'static,
{
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<T>) + 'static,
    {
        let weak = widget.downgrade();

        self.subscribe_widget(widget, move |accessor| {
            if let Some(widget) = weak.upgrade() {
                f(&widget, accessor);
            }
        });
    }
}
