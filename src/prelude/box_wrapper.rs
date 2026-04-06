use gtk::{glib::object::IsA, prelude::BoxExt};

#[derive(Debug)]
pub struct BoxWrapper(pub gtk::Box);

impl BoxWrapper {
    pub fn append_all(self, iter: impl Iterator<Item = impl IsA<gtk::Widget>>) -> Self {
        for widget in iter {
            self.0.append(&widget);
        }
        self
    }
    
    pub fn build(self) -> gtk::Box {
        self.0
    }
}
