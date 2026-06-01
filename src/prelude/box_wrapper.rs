use gtk::{glib::object::IsA, prelude::BoxExt};

use crate::runtime::{Runtime, Scope};

#[derive(Debug, Clone)]
pub struct BoxWrapper(pub gtk::Box);

impl BoxWrapper {
    pub fn new(gtk_box: gtk::Box) -> Self {
        Runtime::get().bind_widget(&gtk_box);
        Self(gtk_box)
    }

    pub fn append_all(self, iter: impl Iterator<Item = impl IsA<gtk::Widget>>) -> Self {
        for widget in iter {
            self.0.append(&widget);
        }
        self
    }

    pub fn append(self, widget: impl IsA<gtk::Widget>) -> Self {
        self.0.append(&widget);

        self
    }

    pub fn build(self) -> gtk::Box {
        self.0
    }

    pub fn children<F>(self, f: F) -> Self
    where
        F: FnOnce(),
    {
        Runtime::get().run_with_scope(self.clone(), f);
        self
    }
}

pub fn hbox() -> BoxWrapper {
    BoxWrapper::new(gtk::Box::new(gtk::Orientation::Horizontal, 0))
}

pub fn vbox() -> BoxWrapper {
    BoxWrapper::new(gtk::Box::new(gtk::Orientation::Vertical, 0))
}

impl Scope for BoxWrapper {
    fn bind_widget(&self, widget: gtk::Widget) {
        self.0.append(&widget);
    }
}
