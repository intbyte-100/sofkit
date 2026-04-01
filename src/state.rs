use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::{any::Any, rc::Weak};

use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{self, Object};

// ─── State trait ───────────────────────────────────────────────────────────────

pub trait State<T: 'static>: Clone {
    fn subscribe<W: Fn(&T) + 'static>(&self, callback: W) -> Option<()>;

    fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()>;

    fn update(&self, value: T) -> Option<()> {
        self.edit(move |it| *it = value)
    }

    fn get(&self) -> Option<Rc<RefCell<StateCell<T>>>>;

    fn map<M: 'static, C>(self, map: C) -> MappedState<Self, T, M, C>
    where
        C: Fn(&T) -> M + Clone + 'static,
        Self: Sized,
    {
        MappedState::new(self, map)
    }
}

// ─── InnerState ────────────────────────────────────────────────────────────────

pub struct StateCell<T> {
    state: T,
    subscribers: Vec<Box<dyn Fn(&T)>>,
}

impl<T> StateCell<T> {
    fn new(state: T) -> Self {
        Self {
            state,
            subscribers: Vec::new(),
        }
    }
}

impl<T> Deref for StateCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

// ─── StateHandle ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct StateHandle<T> {
    inner: Weak<dyn Any>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static> StateHandle<T> {
    pub fn subscribe<W: Fn(&T) + 'static>(&self, callback: W) -> Option<()> {
        self.inner.upgrade().and_then(|rc_any| {
            rc_any
                .downcast::<RefCell<StateCell<T>>>()
                .and_then(|it| {
                    callback(&it.borrow().state);
                    it.borrow_mut().subscribers.push(Box::new(callback));
                    Ok(())
                })
                .ok()
        })
    }

    pub fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()> {
        let result = self.inner.upgrade().and_then(|rc_any| {
            rc_any
                .downcast::<RefCell<StateCell<T>>>()
                .and_then(|it| {
                    {
                        let mut it = it.borrow_mut();
                        callback(&mut it.state);
                    }

                    let it = it.borrow();
                    it.subscribers
                        .iter()
                        .for_each(|subscriber| subscriber(&it.state));
                    Ok(())
                })
                .ok()
        });

        if cfg!(debug_assertions) && result.is_none() {
            eprintln!("Warning: State used without attach_state_holder()");
        }

        result
    }

    pub fn get(&self) -> Option<Rc<RefCell<StateCell<T>>>> {
        self.inner
            .upgrade()
            .and_then(|rc_any| rc_any.downcast::<RefCell<StateCell<T>>>().ok())
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

    fn get(&self) -> Option<Rc<RefCell<StateCell<T>>>> {
        StateHandle::get(self)
    }
}

// ─── MappedState ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct MappedState<S, F: 'static, M: 'static, C>
where
    S: State<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    state: S,
    cached: RefCell<Option<Rc<RefCell<StateCell<M>>>>>,
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
            .map(|it| Rc::new(RefCell::new(StateCell::new((self.map)(&it.borrow().state)))));

        let map = self.map.clone();
        
        self.cached.borrow().as_ref().and_then(|it| {
            let weak = Rc::downgrade(it);

            self.state.subscribe(move |it| {
                if let Some(rc) = weak.upgrade() {
                    rc.borrow_mut().state = (map)(&it);
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
                    callback(&rc.borrow().state);
                })
            }
            None => None,
        }
    }

    fn edit<W: FnOnce(&mut M) + 'static>(&self, _callback: W) -> Option<()> {
        None
    }

    fn get(&self) -> Option<Rc<RefCell<StateCell<M>>>> {
        match self.state.get() {
            Some(_) => self.cached.borrow().clone(),
            None => None,
        }
    }
}

// ─── StateHolder (GObject) ─────────────────────────────────────────────────────

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
        let rc: Rc<dyn Any> = Rc::new(RefCell::new(StateCell::new(value)));
        let weak = Rc::downgrade(&rc);

        self.imp().states.borrow_mut().push(rc);

        StateHandle {
            inner: weak,
            _marker: std::marker::PhantomData,
        }
    }
}
