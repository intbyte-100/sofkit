use std::{cell::RefCell, rc::Rc};

use gtk::glib::object::{Cast, IsA};

pub struct Runtime {
    vec: RefCell<Vec<Box<dyn Scope>>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            vec: Default::default(),
        }
    }

    pub fn get() -> Rc<Runtime> {
        RUNTIME.with(|it| it.clone())
    }

    pub fn run_with_scope<F, S: Scope + 'static>(&self, scope: S, f: F)
    where
        F: FnOnce(),
    {
        self.vec.borrow_mut().push(Box::new(scope));
        f();
        self.vec.borrow_mut().pop();
    }

    pub fn with_current_scope<F>(&self, f: F)
    where
        F: FnOnce(&dyn Scope),
    {
        f(self
            .vec
            .borrow()
            .last()
            .map(|s| s.as_ref())
            .expect("Scope in not registered"))
    }

    pub fn bind_widget(&self, widget: &impl IsA<gtk::Widget>) {
        let widget: gtk::Widget = widget.upcast_ref().clone();
        self.with_current_scope(|scope| scope.bind_widget(widget));
    }
}

thread_local! {
    static RUNTIME: Rc<Runtime> = Rc::from(Runtime::new());
}

pub trait Scope {
    fn bind_widget(&self, widget: gtk::Widget);
}
