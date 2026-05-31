use gtk::glib::object::{IsA, ObjectExt};
use crate::state::{ReadState, StateAccessor};

pub trait ReactiveValue<T> {
    fn bind<W, F>(self, widget: &W, f: F)
    where
        W: IsA<gtk::Widget>,
        F: Fn(&W, &StateAccessor<T>) + 'static;
}

macro_rules! impl_reactive_value_for_self {
    ($($ty:ty),*) => {
        $(
            impl ReactiveValue<$ty> for $ty {
                fn bind<W, F>(self, widget: &W, f: F)
                where
                    W: IsA<gtk::Widget>,
                    F: Fn(&W, &StateAccessor<$ty>) + 'static,
                {
                    let accessor = StateAccessor::new(self);
                    f(widget, &accessor);
                }
            }
        )*
    };
}

macro_rules! impl_reactive_value_for_string_repr {
    ($($ty:ty),*) => {
        $(
            impl ReactiveValue<String> for $ty {
                fn bind<W, F>(self, widget: &W, f: F)
                where
                    W: IsA<gtk::Widget>,
                    F: Fn(&W, &StateAccessor<String>) + 'static,
                {
                    let accessor = StateAccessor::new(self.to_string());
                    f(widget, &accessor);
                }
            }
        )*
    };
}

impl_reactive_value_for_self!(String, i32, bool, f64, f32, usize, isize, u32, i64, u64);
impl_reactive_value_for_string_repr!(&str, &String, i32, bool, f64, f32, usize, isize, u32, i64, u64);


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