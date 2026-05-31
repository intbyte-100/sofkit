use gtk::glib::object::{IsA, ObjectExt};

use crate::state::ReadState;

pub trait ReactiveValue<T> {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &T) + 'static;
}

impl ReactiveValue<String> for String {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &String) {
        f(widget, &self)
    }
}

impl ReactiveValue<String> for &str {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &String) {
        f(widget, &self.to_string())
    }
}

impl ReactiveValue<String> for &String {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &String) {
        f(widget, self)
    }
}

impl ReactiveValue<i32> for i32 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &i32) {
        f(widget, &self)
    }
}

impl ReactiveValue<bool> for bool {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &bool) {
        f(widget, &self)
    }
}

impl ReactiveValue<f64> for f64 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &f64),
    {
        f(widget, &self)
    }
}

impl ReactiveValue<f32> for f32 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &f32),
    {
        f(widget, &self)
    }
}

impl ReactiveValue<usize> for usize {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &usize),
    {
        f(widget, &self)
    }
}

impl ReactiveValue<isize> for isize {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &isize),
    {
        f(widget, &self)
    }
}

impl ReactiveValue<u32> for u32 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &u32),
    {
        f(widget, &self)
    }
}

impl ReactiveValue<i64> for i64 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &i64),
    {
        f(widget, &self)
    }
}

impl ReactiveValue<u64> for u64 {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &u64),
    {
        f(widget, &self)
    }
}

impl<T, S> ReactiveValue<T> for S where S: ReadState<T>,
    T: Clone + 'static,
{
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &T) +'static,
    {
        let weak = widget.downgrade();
        
        self.subscribe_widget(widget, move |accessor| {
            accessor.with(|it| {
                if let Some(widget) = weak.upgrade() {
                    f(&widget, it);
                }
            })
        });
    }
}


