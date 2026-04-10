use crate::prelude::state_ext::StateHolderExt;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::{any::Any, rc::Weak};

use gtk::Widget;
use gtk::glib::object::IsA;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{self, Object};

use crate::reactive_frame::current_reactive_frame;

pub trait State<T: 'static>: Clone {
    fn subscribe<W: Fn(&T) + 'static>(&self, callback: W) -> Option<Subscription>;

    #[inline]
    fn subscribe_widget<W: Fn(&T) + 'static>(
        &self,
        widget: &impl IsA<Widget>,
        callback: W,
    ) -> Option<()> {
        self.subscribe(callback)
            .map(|s| widget.attach_subscription(s))
    }

    fn with<W: FnOnce(&T) -> D, D>(&self, callback: W) -> Option<D>;

    fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()>;

    fn update(&self, value: T) -> Option<()> {
        self.edit(move |it| *it = value)
    }

    fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.with(|it| it.clone())
    }

    fn map<M: 'static, C>(&self, map: C) -> MappedState<Self, T, M, C>
    where
        C: Fn(&T) -> M + Clone + 'static,
        Self: Sized,
    {
        MappedState::new(self.clone(), map)
    }
}

pub struct StateCell<T> {
    scheduled_frame: Cell<u64>,
    state: RefCell<T>,
    subscribers: RefCell<HashMap<i32, Box<dyn Fn(&T)>>>,
}

impl<T> StateCell<T> {
    fn new(state: T) -> Self {
        Self {
            scheduled_frame: Cell::new(0),
            state: RefCell::new(state),
            subscribers: RefCell::default(),
        }
    }

    fn needs_subscription_update(&self) -> bool {
        if self.scheduled_frame.get() != current_reactive_frame() {
            self.scheduled_frame.set(current_reactive_frame());
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

thread_local! {
    static SUBSCRIPTION_ID: Cell<u32> = Cell::default();
}

fn new_subscription_id() -> u32 {
    SUBSCRIPTION_ID.with(|it| {
        let id = it.get();
        it.set(id + 1);
        id
    })
}

#[derive(Clone)]
pub struct StateHandle<T> {
    inner: Weak<StateCell<T>>,
}

impl<T: 'static + Clone> StateHandle<T> {
    fn subscribe<W: Fn(&T) + 'static>(&self, callback: W) -> Option<Subscription> {
        let weak = self.inner.clone();
        self.inner.upgrade().map(|it| {
            callback(&it.borrow());

            let id = new_subscription_id() as i32;
            it.subscribers.borrow_mut().insert(id, Box::new(callback));

            Subscription::new(Box::new(move || {
                if let Some(it) = weak.upgrade() {
                    it.subscribers.borrow_mut().remove(&id);
                }
            }))
        })
    }

    fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()> {
        let result = {
            self.inner.upgrade().map(|it| {
                callback(&mut it.state.borrow_mut());

                if it.needs_subscription_update() {
                    glib::idle_add_local_once(move || {
                        let value = it.state.borrow().clone();

                        let mut subscribers =
                            std::mem::take::<HashMap<_, _>>(&mut it.subscribers.borrow_mut());

                        for subscriber in subscribers.values() {
                            subscriber(&value);
                        }

                        let mut_ref = &mut it.subscribers.borrow_mut();

                        std::mem::swap(&mut subscribers, mut_ref);

                        mut_ref.extend(subscribers);
                    });
                }
                ()
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
    fn subscribe<W: Fn(&T) + 'static>(&self, callback: W) -> Option<Subscription> {
        StateHandle::subscribe(self, callback)
    }

    fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()> {
        StateHandle::edit(self, callback)
    }

    fn with<W: FnOnce(&T) -> D, D>(&self, callback: W) -> Option<D> {
        self.inner
            .upgrade()
            .map(|cell| callback(&cell.state.borrow()))
    }
}

struct InnerMappedState<S, F: 'static, M: 'static, C>
where
    S: State<F> + 'static,
    C: Fn(&F) -> M + Clone + 'static,
{
    state: S,
    cached: RefCell<Option<M>>,
    reactive_frame: Cell<u64>,
    map: C,
    _marker: std::marker::PhantomData<(F, M)>,
}

impl<S, F: 'static, M: 'static, C> InnerMappedState<S, F, M, C>
where
    S: State<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    fn new(state: S, map: C) -> Rc<Self> {
        Self {
            cached: state.with(|it| map(it)).into(),
            state,
            reactive_frame: Default::default(),
            map,
            _marker: std::marker::PhantomData,
        }
        .into()
    }

    fn apply_map(&self, value: &F) {
        if self.reactive_frame.get() != current_reactive_frame() {
            self.cached.replace(Some((self.map)(value)));
            self.reactive_frame.set(current_reactive_frame());
        }
    }

    fn subscribe<W: Fn(&M) + 'static>(self: Rc<Self>, callback: W) -> Option<Subscription> {
        let cloned = self.clone();
        self.state.subscribe(move |value| {
            cloned.apply_map(value);
            callback(cloned.cached.borrow().as_ref().unwrap());
        })
    }

    fn with<W: FnOnce(&M) -> D, D>(self: Rc<Self>, callback: W) -> Option<D> {
        self.cached.borrow().as_ref().map(|it| callback(it))
    }
}

#[derive(Clone)]
pub struct MappedState<S, F: 'static, M: 'static, C>
where
    S: State<F> + 'static,
    C: Fn(&F) -> M + Clone + 'static,
{
    inner: Rc<InnerMappedState<S, F, M, C>>,
}

impl<S, F: 'static, M: 'static, C> MappedState<S, F, M, C>
where
    S: State<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    pub fn new(state: S, map: C) -> Self {
        Self {
            inner: InnerMappedState::new(state, map),
        }
    }
}

impl<S, F: 'static + Clone, M: 'static + Clone, C> State<M> for MappedState<S, F, M, C>
where
    S: State<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    fn subscribe<W: Fn(&M) + 'static>(&self, callback: W) -> Option<Subscription> {
        self.inner.clone().subscribe(callback)
    }

    fn edit<W: FnOnce(&mut M) + 'static>(&self, _callback: W) -> Option<()> {
        None
    }

    fn with<W: FnOnce(&M) -> D, D>(&self, callback: W) -> Option<D> {
        self.inner.clone().with(callback)
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

pub struct Subscription {
    on_drop: Option<Box<dyn FnOnce()>>,
}

impl Subscription {
    pub fn new(on_drop: Box<dyn FnOnce()>) -> Self {
        Self {
            on_drop: Some(on_drop),
        }
    }

    pub fn unsubscribe(&mut self) {
        if let Some(unsubscribe) = std::mem::take(&mut self.on_drop) {
            glib::idle_add_local_once(unsubscribe);
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

pub struct SubscriptionHolder {
    subscription: RefCell<Vec<Subscription>>,
}

impl SubscriptionHolder {
    pub fn new() -> Self {
        Self {
            subscription: Default::default(),
        }
    }

    pub fn attach_subscription(&self, subscription: Subscription) {
        self.subscription.borrow_mut().push(subscription);
    }
}
