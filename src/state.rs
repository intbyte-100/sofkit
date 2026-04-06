use std::cell::{Cell, Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::{any::Any, rc::Weak};

use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{self, Object};

use crate::app::current_frame;


pub trait State<T: 'static>: Clone {
    fn subscribe<W: Fn(&T) + 'static>(&self, callback: W) -> Option<()>;

    fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()>;

    fn update(&self, value: T) -> Option<()> {
        self.edit(move |it| *it = value)
    }

    fn get(&self) -> Option<Rc<StateCell<T>>>;

    fn map<M: 'static, C>(&self, map: C) -> MappedState<Self, T, M, C>
    where
        C: Fn(&T) -> M + Clone + 'static,
        Self: Sized,
    {
        MappedState::new(self.clone(), map)
    }
}


pub struct StateCell<T> {
    scheculed_frame: Cell<u64>,

    state: RefCell<T>,
    subscribers: RefCell<Vec<Box<dyn Fn(&T)>>>,
}

impl<T> StateCell<T> {
    fn new(state: T) -> Self {
        Self {
            scheculed_frame: Cell::new(0),
            state: RefCell::new(state),
            subscribers: RefCell::default(),
        }
    }

    fn needs_subscription_update(&self) -> bool {
        if self.scheculed_frame.get() != current_frame() {
            self.scheculed_frame.set(current_frame());
            true
        } else {
            false
        }
    }
}

impl<T> Deref for StateCell<T> {
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}


#[derive(Clone)]
pub struct StateHandle<T> {
    inner: Weak<StateCell<T>>,
}

impl<T: 'static + Clone> StateHandle<T> {
    pub fn subscribe<W: Fn(&T) + 'static>(&self, callback: W) -> Option<()> {
        self.inner.upgrade().and_then(|it| {
            callback(&it.borrow());
            it.subscribers.borrow_mut().push(Box::new(callback));
            Some(())
        })
    }

    pub fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()> {
        let result = {
            self.inner.upgrade().and_then(|it| {
                callback(&mut it.state.borrow_mut());

                if it.needs_subscription_update() {
                    glib::idle_add_local_once(move || {
                        let value = it.state.borrow().clone();
                        
                        it.subscribers
                            .borrow()
                            .iter()
                            .for_each(|subscriber| subscriber(&value));
                    });
                }
                Some(())
            })
        };

        if cfg!(debug_assertions) && result.is_none() {
            eprintln!("Warning: State used without attach_state_holder()");
        }

        result
    }

    pub fn get(&self) -> Option<Rc<StateCell<T>>> {
        self.inner.upgrade()
    }

    pub fn set(&self, value: T) -> Option<()> {
        self.edit(move |it| *it = value)
    }
}

impl<T: 'static + Clone> State<T> for StateHandle<T> {
    fn subscribe<W: Fn(&T) + 'static>(&self, callback: W) -> Option<()> {
        StateHandle::subscribe(self, callback)
    }

    fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()> {
        StateHandle::edit(self, callback)
    }

    fn get(&self) -> Option<Rc<StateCell<T>>> {
        StateHandle::get(self)
    }
}


#[derive(Clone)]
pub struct MappedState<S, F: 'static, M: 'static, C>
where
    S: State<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    state: S,
    cached: RefCell<Option<Rc<StateCell<M>>>>,
    map: C,
    _marker: std::marker::PhantomData<(F, M)>,
}

impl<S, F: 'static, M: 'static, C> MappedState<S, F, M, C>
where
    S: State<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    pub fn new(state: S, map: C) -> Self {
        let mapped = Self {
            state,
            cached: RefCell::new(None),
            map,
            _marker: std::marker::PhantomData,
        };

        mapped.apply_map();

        mapped
    }

    fn apply_map(&self) {
        *self.cached.borrow_mut() = self
            .state
            .get()
            .map(|it| Rc::new(StateCell::new((self.map)(&it.borrow()))));

        let map = self.map.clone();

        self.cached.borrow().as_ref().and_then(|it| {
            let weak = Rc::downgrade(it);

            self.state.subscribe(move |it| {
                if let Some(rc) = weak.upgrade() {
                    *rc.borrow_mut() = (map)(&it);
                }
            });
            Some(())
        });
    }
}

impl<S, F: 'static + Clone, M: 'static + Clone, C> State<M> for MappedState<S, F, M, C>
where
    S: State<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    fn subscribe<W: Fn(&M) + 'static>(&self, callback: W) -> Option<()> {
        let cached = self.cached.borrow();

        match cached.as_ref() {
            Some(rc) => {
                let rc = rc.clone();

                self.state.subscribe(move |value| {
                    callback(&rc.borrow());
                })
            }
            None => None,
        }
    }

    fn edit<W: FnOnce(&mut M) + 'static>(&self, _callback: W) -> Option<()> {
        None
    }

    fn get(&self) -> Option<Rc<StateCell<M>>> {
        match self.state.get() {
            Some(_) => self.cached.borrow().clone(),
            None => None,
        }
    }
}


mod imp {
    use std::rc::Rc;

    use gtk::glib::subclass::{object::ObjectImpl, types::ObjectSubclass};

    use super::*;

    #[derive(Default)]
    pub struct StateHolder {
        pub states: RefCell<Vec<Rc<dyn Any>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StateHolder {
        const NAME: &'static str = "StateHolder";
        type Type = super::StateHolder;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for StateHolder {}
}

glib::wrapper! {
    pub struct StateHolder(ObjectSubclass<imp::StateHolder>);
}

impl StateHolder {
    pub fn new() -> Self {
        Object::new()
    }

    pub fn state<T: 'static>(&self, value: T) -> StateHandle<T> {
        let rc = Rc::new(StateCell::new(value));
        let weak = Rc::downgrade(&rc);

        self.imp().states.borrow_mut().push(rc);

        StateHandle { inner: weak }
    }
}
